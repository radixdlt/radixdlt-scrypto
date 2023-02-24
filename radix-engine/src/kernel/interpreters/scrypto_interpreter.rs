use crate::blueprints::access_controller::AccessControllerNativePackage;
use crate::blueprints::account::AccountNativePackage;
use crate::blueprints::clock::ClockNativePackage;
use crate::blueprints::epoch_manager::EpochManagerNativePackage;
use crate::blueprints::identity::IdentityNativePackage;
use crate::blueprints::logger::LoggerNativePackage;
use crate::blueprints::resource::ResourceManagerNativePackage;
use crate::blueprints::transaction_runtime::TransactionRuntimeNativePackage;
use crate::errors::ScryptoFnResolvingError;
use crate::errors::{InterpreterError, KernelError, RuntimeError};
use crate::kernel::actor::{ResolvedActor, ResolvedReceiver};
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::{
    ExecutableInvocation, Executor, KernelNodeApi, KernelSubstateApi, KernelWasmApi,
    TemporaryResolvedInvocation,
};
use crate::system::global::GlobalSubstate;
use crate::system::node_modules::access_rules::{AccessRulesNativePackage, AuthZoneNativePackage};
use crate::system::node_modules::metadata::MetadataNativePackage;
use crate::system::node_modules::royalty::RoyaltyNativePackage;
use crate::system::package::Package;
use crate::system::type_info::PackageCodeTypeSubstate;
use crate::types::*;
use crate::wasm::{WasmEngine, WasmInstance, WasmInstrumenter, WasmMeteringConfig, WasmRuntime};
use radix_engine_interface::api::component::TypeInfoSubstate;
use radix_engine_interface::api::node_modules::auth::ACCESS_RULES_BLUEPRINT;
use radix_engine_interface::api::node_modules::metadata::METADATA_BLUEPRINT;
use radix_engine_interface::api::node_modules::royalty::{
    COMPONENT_ROYALTY_BLUEPRINT, PACKAGE_ROYALTY_BLUEPRINT,
};
use radix_engine_interface::api::package::*;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::types::FunctionInvocation;
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::data::*;
use radix_engine_interface::data::{match_schema_with_value, ScryptoValue};

use super::ScryptoRuntime;

impl ExecutableInvocation for MethodInvocation {
    type Exec = ScryptoExecutor;

    fn resolve<D: KernelSubstateApi>(
        self,
        api: &mut D,
    ) -> Result<TemporaryResolvedInvocation<Self::Exec>, RuntimeError> {
        let (_, value, nodes_to_move, mut node_refs_to_copy) =
            IndexedScryptoValue::from_slice(&self.args)
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?
                .unpack();

        let (package_address, blueprint_name) = match self.receiver.1 {
            NodeModuleId::SELF => {
                let handle = api.kernel_lock_substate(
                    self.receiver.0,
                    NodeModuleId::TypeInfo,
                    SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
                    LockFlags::read_only(),
                )?;
                let type_info: &TypeInfoSubstate = api.kernel_get_substate_ref(handle)?;
                let object_info = (type_info.package_address, type_info.blueprint_name.clone());
                api.kernel_drop_lock(handle)?;

                object_info
            }
            NodeModuleId::Metadata => {
                // TODO: Check if type has metadata
                (METADATA_PACKAGE, METADATA_BLUEPRINT.to_string())
            }
            NodeModuleId::ComponentRoyalty => {
                // TODO: Check if type has royalty
                (ROYALTY_PACKAGE, COMPONENT_ROYALTY_BLUEPRINT.to_string())
            }
            NodeModuleId::PackageRoyalty => {
                // TODO: Check if type has royalty
                (ROYALTY_PACKAGE, PACKAGE_ROYALTY_BLUEPRINT.to_string())
            }
            NodeModuleId::AccessRules | NodeModuleId::AccessRules1 => {
                // TODO: Check if type has access ruls
                (ACCESS_RULES_PACKAGE, ACCESS_RULES_BLUEPRINT.to_string())
            }
            _ => todo!(),
        };

        // Deref if global
        let resolved_receiver = if let RENodeId::Global(..) = self.receiver.0 {
            let handle = api.kernel_lock_substate(
                self.receiver.0,
                NodeModuleId::SELF,
                SubstateOffset::Global(GlobalOffset::Global),
                LockFlags::empty(),
            )?;
            let derefed: &GlobalSubstate = api.kernel_get_substate_ref(handle)?;
            let derefed = derefed.node_deref();
            ResolvedReceiver::derefed(
                MethodReceiver(derefed, self.receiver.1),
                self.receiver.0,
                handle,
            )
        } else {
            ResolvedReceiver::new(self.receiver)
        };

        // Pass the component ref
        node_refs_to_copy.insert(resolved_receiver.receiver.0);

        let fn_identifier = FnIdentifier::new(
            package_address,
            blueprint_name.clone(),
            self.fn_name.clone(),
        );
        let actor = ResolvedActor::method(fn_identifier.clone(), resolved_receiver);

        let code_type = if package_address.eq(&PACKAGE_LOADER) {
            // TODO: Remove this weirdness
            node_refs_to_copy.insert(RENodeId::Global(Address::Resource(RADIX_TOKEN)));
            PackageCodeTypeSubstate::Precompiled
        } else {
            let handle = api.kernel_lock_substate(
                RENodeId::Global(Address::Package(package_address)),
                NodeModuleId::SELF,
                SubstateOffset::Package(PackageOffset::CodeType),
                LockFlags::read_only(),
            )?;
            let code_type: &PackageCodeTypeSubstate = api.kernel_get_substate_ref(handle)?;
            let code_type = code_type.clone();
            api.kernel_drop_lock(handle)?;
            code_type
        };

        let export_name = match code_type {
            PackageCodeTypeSubstate::Precompiled => {
                // TODO: Do we need to check against the abi? Probably not since we should be able to verify this
                // TODO: in the native package itself.
                self.fn_name.to_string() // TODO: Clean this up
            }
            PackageCodeTypeSubstate::Wasm => {
                node_refs_to_copy.insert(RENodeId::Global(Address::Component(EPOCH_MANAGER)));
                node_refs_to_copy.insert(RENodeId::Global(Address::Component(CLOCK)));
                node_refs_to_copy.insert(RENodeId::Global(Address::Resource(RADIX_TOKEN)));
                node_refs_to_copy.insert(RENodeId::Global(Address::Resource(PACKAGE_TOKEN)));
                node_refs_to_copy
                    .insert(RENodeId::Global(Address::Resource(ECDSA_SECP256K1_TOKEN)));
                node_refs_to_copy.insert(RENodeId::Global(Address::Resource(EDDSA_ED25519_TOKEN)));

                let package_global = RENodeId::Global(Address::Package(package_address));
                let handle = api.kernel_lock_substate(
                    package_global,
                    NodeModuleId::SELF,
                    SubstateOffset::Package(PackageOffset::Info),
                    LockFlags::read_only(),
                )?;
                let info: &PackageInfoSubstate = api.kernel_get_substate_ref(handle)?;
                for dependent_resource in &info.dependent_resources {
                    node_refs_to_copy
                        .insert(RENodeId::Global(Address::Resource(*dependent_resource)));
                }

                // Find the abi
                let abi =
                    info.blueprint_abi(&blueprint_name)
                        .ok_or(RuntimeError::InterpreterError(
                            InterpreterError::InvalidScryptoInvocation(
                                fn_identifier.clone(),
                                ScryptoFnResolvingError::BlueprintNotFound,
                            ),
                        ))?;
                let fn_abi =
                    abi.get_fn_abi(&self.fn_name)
                        .ok_or(RuntimeError::InterpreterError(
                            InterpreterError::InvalidScryptoInvocation(
                                fn_identifier.clone(),
                                ScryptoFnResolvingError::MethodNotFound,
                            ),
                        ))?;

                if !fn_abi.mutability.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::InvalidInvocation,
                    ));
                }

                if !match_schema_with_value(&fn_abi.input, &value) {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoInvocation(
                            fn_identifier.clone(),
                            ScryptoFnResolvingError::InvalidInput,
                        ),
                    ));
                }

                let export_name = fn_abi.export_name.clone();
                api.kernel_drop_lock(handle)?;

                export_name
            }
        };

        let executor = ScryptoExecutor {
            package_address,
            export_name,
            receiver: Some(resolved_receiver.receiver),
        };

        // TODO: remove? currently needed for `Runtime::package_address()` API.
        node_refs_to_copy.insert(RENodeId::Global(Address::Package(package_address)));

        let resolved = TemporaryResolvedInvocation {
            resolved_actor: actor,
            update: CallFrameUpdate {
                nodes_to_move,
                node_refs_to_copy,
            },
            executor,
            args: value,
        };

        Ok(resolved)
    }
}

impl ExecutableInvocation for FunctionInvocation {
    type Exec = ScryptoExecutor;

    fn resolve<D: KernelSubstateApi>(
        self,
        api: &mut D,
    ) -> Result<TemporaryResolvedInvocation<Self::Exec>, RuntimeError> {
        let (_, value, nodes_to_move, mut node_refs_to_copy) =
            IndexedScryptoValue::from_slice(&self.args)
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?
                .unpack();

        let actor = ResolvedActor::function(self.fn_identifier.clone());

        let code_type = if self.fn_identifier.package_address.eq(&PACKAGE_LOADER) {
            // TODO: Remove this weirdness
            node_refs_to_copy.insert(RENodeId::Global(Address::Resource(RADIX_TOKEN)));
            PackageCodeTypeSubstate::Precompiled
        } else {
            let handle = api.kernel_lock_substate(
                RENodeId::Global(Address::Package(self.fn_identifier.package_address)),
                NodeModuleId::SELF,
                SubstateOffset::Package(PackageOffset::CodeType),
                LockFlags::read_only(),
            )?;
            let code_type: &PackageCodeTypeSubstate = api.kernel_get_substate_ref(handle)?;
            let code_type = code_type.clone();
            api.kernel_drop_lock(handle)?;
            code_type
        };

        let export_name = match code_type {
            PackageCodeTypeSubstate::Precompiled => {
                // TODO: Do we need to check against the abi? Probably not since we should be able to verify this
                // TODO: in the native package itself.
                self.fn_identifier.ident.to_string() // TODO: Clean this up
            }
            PackageCodeTypeSubstate::Wasm => {
                node_refs_to_copy.insert(RENodeId::Global(Address::Component(EPOCH_MANAGER)));
                node_refs_to_copy.insert(RENodeId::Global(Address::Component(CLOCK)));
                node_refs_to_copy.insert(RENodeId::Global(Address::Resource(RADIX_TOKEN)));
                node_refs_to_copy.insert(RENodeId::Global(Address::Resource(PACKAGE_TOKEN)));
                node_refs_to_copy
                    .insert(RENodeId::Global(Address::Resource(ECDSA_SECP256K1_TOKEN)));
                node_refs_to_copy.insert(RENodeId::Global(Address::Resource(EDDSA_ED25519_TOKEN)));

                let package_global =
                    RENodeId::Global(Address::Package(self.fn_identifier.package_address));
                let handle = api.kernel_lock_substate(
                    package_global,
                    NodeModuleId::SELF,
                    SubstateOffset::Package(PackageOffset::Info),
                    LockFlags::read_only(),
                )?;
                let info: &PackageInfoSubstate = api.kernel_get_substate_ref(handle)?;
                for dependent_resource in &info.dependent_resources {
                    node_refs_to_copy
                        .insert(RENodeId::Global(Address::Resource(*dependent_resource)));
                }

                // Find the abi
                let abi = info
                    .blueprint_abi(&self.fn_identifier.blueprint_name)
                    .ok_or(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoInvocation(
                            self.fn_identifier.clone(),
                            ScryptoFnResolvingError::BlueprintNotFound,
                        ),
                    ))?;
                let fn_abi = abi.get_fn_abi(&self.fn_identifier.ident).ok_or(
                    RuntimeError::InterpreterError(InterpreterError::InvalidScryptoInvocation(
                        self.fn_identifier.clone(),
                        ScryptoFnResolvingError::MethodNotFound,
                    )),
                )?;

                if fn_abi.mutability.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::InvalidInvocation,
                    ));
                }

                if !match_schema_with_value(&fn_abi.input, &value) {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoInvocation(
                            self.fn_identifier.clone(),
                            ScryptoFnResolvingError::InvalidInput,
                        ),
                    ));
                }

                let export_name = fn_abi.export_name.clone();
                api.kernel_drop_lock(handle)?;

                export_name
            }
        };

        let executor = ScryptoExecutor {
            package_address: self.fn_identifier.package_address,
            export_name,
            receiver: None,
        };

        // TODO: remove? currently needed for `Runtime::package_address()` API.
        node_refs_to_copy.insert(RENodeId::Global(Address::Package(
            self.fn_identifier.package_address,
        )));

        let resolved = TemporaryResolvedInvocation {
            resolved_actor: actor,
            update: CallFrameUpdate {
                nodes_to_move,
                node_refs_to_copy,
            },
            args: value,
            executor,
        };

        Ok(resolved)
    }
}

pub struct ScryptoExecutor {
    pub package_address: PackageAddress,
    pub export_name: String,
    pub receiver: Option<MethodReceiver>,
}

impl Executor for ScryptoExecutor {
    type Output = ScryptoValue;

    fn execute<Y, W>(
        self,
        args: ScryptoValue,
        api: &mut Y,
    ) -> Result<(ScryptoValue, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + KernelWasmApi<W> + ClientApi<RuntimeError>,
        W: WasmEngine,
    {
        let output = if self.package_address.eq(&PACKAGE_LOADER) {
            NativeVm::invoke_native_package(
                NATIVE_PACKAGE_CODE_ID,
                self.receiver,
                &self.export_name,
                args,
                api,
            )?
        } else {
            // Make dependent resources/components visible
            let handle = api.kernel_lock_substate(
                RENodeId::Global(Address::Package(self.package_address)),
                NodeModuleId::SELF,
                SubstateOffset::Package(PackageOffset::Info),
                LockFlags::read_only(),
            )?;
            api.kernel_drop_lock(handle)?;

            let code_type = {
                let handle = api.kernel_lock_substate(
                    RENodeId::Global(Address::Package(self.package_address)),
                    NodeModuleId::SELF,
                    SubstateOffset::Package(PackageOffset::CodeType),
                    LockFlags::read_only(),
                )?;
                let code_type: &PackageCodeTypeSubstate = api.kernel_get_substate_ref(handle)?;
                let code_type = code_type.clone();
                api.kernel_drop_lock(handle)?;
                code_type
            };

            let output = match code_type {
                PackageCodeTypeSubstate::Precompiled => {
                    let handle = api.kernel_lock_substate(
                        RENodeId::Global(Address::Package(self.package_address)),
                        NodeModuleId::SELF,
                        SubstateOffset::Package(PackageOffset::Code),
                        LockFlags::read_only(),
                    )?;
                    let code: &PackageCodeSubstate = api.kernel_get_substate_ref(handle)?;
                    let native_package_code_id = code.code[0];
                    api.kernel_drop_lock(handle)?;
                    NativeVm::invoke_native_package(
                        native_package_code_id,
                        self.receiver,
                        &self.export_name,
                        args,
                        api,
                    )?
                }
                PackageCodeTypeSubstate::Wasm => {
                    let rtn_type = {
                        let handle = api.kernel_lock_substate(
                            RENodeId::Global(Address::Package(self.package_address)),
                            NodeModuleId::SELF,
                            SubstateOffset::Package(PackageOffset::Info),
                            LockFlags::read_only(),
                        )?;
                        let package_info: &PackageInfoSubstate =
                            api.kernel_get_substate_ref(handle)?;
                        let fn_abi = package_info
                            .fn_abi(&self.export_name)
                            .expect("TODO: Remove this expect");
                        let rtn_type = fn_abi.output.clone();
                        api.kernel_drop_lock(handle)?;
                        rtn_type
                    };

                    let mut instance = {
                        let handle = api.kernel_lock_substate(
                            RENodeId::Global(Address::Package(self.package_address)),
                            NodeModuleId::SELF,
                            SubstateOffset::Package(PackageOffset::Code),
                            LockFlags::read_only(),
                        )?;
                        let wasm_instance =
                            api.kernel_create_wasm_instance(self.package_address, handle)?;
                        api.kernel_drop_lock(handle)?;

                        wasm_instance
                    };

                    let output = {
                        let mut runtime: Box<dyn WasmRuntime> = Box::new(ScryptoRuntime::new(api));

                        let mut input = Vec::new();
                        if let Some(MethodReceiver(component_id, _)) = self.receiver {
                            let component_id: ComponentId = component_id.into();
                            input.push(
                                runtime
                                    .allocate_buffer(
                                        scrypto_encode(&component_id)
                                            .expect("Failed to encode component id"),
                                    )
                                    .expect("Failed to allocate buffer"),
                            );
                        }
                        input.push(
                            runtime
                                .allocate_buffer(
                                    scrypto_encode(&args).expect("Failed to encode args"),
                                )
                                .expect("Failed to allocate buffer"),
                        );

                        instance.invoke_export(&self.export_name, input, &mut runtime)?
                    };
                    let output = IndexedScryptoValue::from_vec(output).map_err(|e| {
                        RuntimeError::InterpreterError(InterpreterError::InvalidScryptoReturn(e))
                    })?;

                    if !match_schema_with_value(&rtn_type, output.as_value()) {
                        return Err(RuntimeError::KernelError(
                            KernelError::InvalidScryptoFnOutput,
                        ));
                    }

                    api.update_wasm_memory_usage(instance.consumed_memory()?)?;

                    output
                }
            };

            output
        };

        let (_, value, nodes_to_move, refs_to_copy) = output.unpack();
        let update = CallFrameUpdate {
            node_refs_to_copy: refs_to_copy,
            nodes_to_move,
        };

        Ok((value, update))
    }
}

struct NativeVm;

impl NativeVm {
    pub fn invoke_native_package<Y>(
        native_package_code_id: u8,
        receiver: Option<MethodReceiver>,
        export_name: &str,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let receiver = receiver.map(|r| r.0);

        match native_package_code_id {
            NATIVE_PACKAGE_CODE_ID => Package::invoke_export(&export_name, receiver, input, api),
            RESOURCE_MANAGER_PACKAGE_CODE_ID => {
                ResourceManagerNativePackage::invoke_export(&export_name, receiver, input, api)
            }
            EPOCH_MANAGER_PACKAGE_CODE_ID => {
                EpochManagerNativePackage::invoke_export(&export_name, receiver, input, api)
            }
            IDENTITY_PACKAGE_CODE_ID => {
                IdentityNativePackage::invoke_export(&export_name, receiver, input, api)
            }
            CLOCK_PACKAGE_CODE_ID => {
                ClockNativePackage::invoke_export(&export_name, receiver, input, api)
            }
            ACCOUNT_PACKAGE_CODE_ID => {
                AccountNativePackage::invoke_export(&export_name, receiver, input, api)
            }
            ACCESS_CONTROLLER_PACKAGE_CODE_ID => {
                AccessControllerNativePackage::invoke_export(&export_name, receiver, input, api)
            }
            LOGGER_CODE_ID => {
                LoggerNativePackage::invoke_export(&export_name, receiver, input, api)
            }
            TRANSACTION_RUNTIME_CODE_ID => {
                TransactionRuntimeNativePackage::invoke_export(&export_name, receiver, input, api)
            }
            AUTH_ZONE_CODE_ID => {
                AuthZoneNativePackage::invoke_export(&export_name, receiver, input, api)
            }
            METADATA_CODE_ID => {
                MetadataNativePackage::invoke_export(&export_name, receiver, input, api)
            }
            ROYALTY_CODE_ID => {
                RoyaltyNativePackage::invoke_export(&export_name, receiver, input, api)
            }
            ACCESS_RULES_CODE_ID => {
                AccessRulesNativePackage::invoke_export(&export_name, receiver, input, api)
            }
            _ => Err(RuntimeError::InterpreterError(
                InterpreterError::NativeInvalidCodeId(native_package_code_id),
            )),
        }
    }
}

pub struct ScryptoInterpreter<W: WasmEngine> {
    pub wasm_engine: W,
    /// WASM Instrumenter
    pub wasm_instrumenter: WasmInstrumenter,
    /// WASM metering config
    pub wasm_metering_config: WasmMeteringConfig,
}

impl<W: WasmEngine + Default> Default for ScryptoInterpreter<W> {
    fn default() -> Self {
        Self {
            wasm_engine: W::default(),
            wasm_instrumenter: WasmInstrumenter::default(),
            wasm_metering_config: WasmMeteringConfig::default(),
        }
    }
}

impl<W: WasmEngine> ScryptoInterpreter<W> {
    pub fn create_instance(&self, package_address: PackageAddress, code: &[u8]) -> W::WasmInstance {
        let instrumented_code =
            self.wasm_instrumenter
                .instrument(package_address, code, self.wasm_metering_config);
        self.wasm_engine.instantiate(&instrumented_code)
    }
}

#[cfg(test)]
mod tests {
    const _: () = {
        fn assert_sync<T: Sync>() {}

        fn assert_all() {
            // The ScryptoInterpreter struct captures the code and module template caches.
            // We therefore share a ScryptoInterpreter as a shared cache across Engine runs on the node.
            // This allows EG multiple mempool submission validations via the Core API at the same time
            // This test ensures the requirement for this cache to be Sync isn't broken
            // (At least when we compile with std, as the node does)
            #[cfg(not(feature = "alloc"))]
            assert_sync::<
                crate::kernel::interpreters::ScryptoInterpreter<crate::wasm::DefaultWasmEngine>,
            >();
        }
    };
}
