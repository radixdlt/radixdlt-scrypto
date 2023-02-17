use crate::blueprints::access_controller::AccessControllerNativePackage;
use crate::blueprints::account::AccountNativePackage;
use crate::blueprints::clock::ClockNativePackage;
use crate::blueprints::epoch_manager::EpochManagerNativePackage;
use crate::blueprints::identity::IdentityNativePackage;
use crate::blueprints::logger::LoggerNativePackage;
use crate::blueprints::resource::ResourceManagerNativePackage;
use crate::blueprints::transaction_processor::TransactionProcessorError;
use crate::blueprints::transaction_runtime::TransactionRuntimeNativePackage;
use crate::errors::{ApplicationError, ScryptoFnResolvingError};
use crate::errors::{InterpreterError, KernelError, RuntimeError};
use crate::kernel::actor::{ResolvedActor, ResolvedReceiver};
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::{
    ExecutableInvocation, Executor, KernelNodeApi, KernelSubstateApi, KernelWasmApi, LockFlags,
};
use crate::system::node_modules::auth::AuthZoneNativePackage;
use crate::system::type_info::TypeInfoSubstate;
use crate::types::*;
use crate::wasm::{WasmEngine, WasmInstance, WasmInstrumenter, WasmMeteringConfig, WasmRuntime};
use radix_engine_interface::api::package::*;
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::api::types::{ScryptoInvocation, ScryptoReceiver};
use radix_engine_interface::api::{
    ClientActorApi, ClientApi, ClientComponentApi, ClientNativeInvokeApi, ClientNodeApi,
    ClientSubstateApi, ClientUnsafeApi,
};
use radix_engine_interface::api::{ClientDerefApi, ClientPackageApi};
use radix_engine_interface::data::*;
use radix_engine_interface::data::{match_schema_with_value, ScryptoValue};

use super::ScryptoRuntime;

impl ExecutableInvocation for ScryptoInvocation {
    type Exec = ScryptoExecutor;

    fn resolve<D: ClientDerefApi<RuntimeError> + KernelSubstateApi>(
        self,
        api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut node_refs_to_copy = HashSet::new();
        let args = IndexedScryptoValue::from_slice(&self.args).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                TransactionProcessorError::InvalidCallData(e),
            ))
        })?;

        let nodes_to_move = args
            .owned_node_ids()
            .map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                    TransactionProcessorError::ReadOwnedNodesError(e),
                ))
            })?
            .into_iter()
            .collect();
        for global_address in args.global_references() {
            node_refs_to_copy.insert(RENodeId::Global(global_address));
        }

        let scrypto_fn_ident = ScryptoFnIdentifier::new(
            self.package_address,
            self.blueprint_name.clone(),
            self.fn_name.clone(),
        );

        let (receiver, actor) = if let Some(receiver) = self.receiver {
            let original_node_id = match receiver {
                ScryptoReceiver::Global(component_address) => {
                    RENodeId::Global(Address::Component(component_address))
                }
                ScryptoReceiver::Resource(resource_address) => {
                    RENodeId::Global(Address::Resource(resource_address))
                }
                ScryptoReceiver::Component(component_id) => RENodeId::Component(component_id),
                ScryptoReceiver::Vault(vault_id) => RENodeId::Vault(vault_id),
                ScryptoReceiver::Bucket(bucket_id) => RENodeId::Bucket(bucket_id),
                ScryptoReceiver::Proof(proof_id) => RENodeId::Proof(proof_id),
                ScryptoReceiver::Worktop => RENodeId::Worktop,
                ScryptoReceiver::Logger => RENodeId::Logger,
                ScryptoReceiver::TransactionRuntime => RENodeId::TransactionRuntime,
                ScryptoReceiver::AuthZoneStack => RENodeId::AuthZoneStack,
            };

            // Type Check
            {
                let handle = api.kernel_lock_substate(
                    original_node_id,
                    NodeModuleId::ComponentTypeInfo,
                    SubstateOffset::ComponentTypeInfo(ComponentTypeInfoOffset::TypeInfo),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.kernel_get_substate_ref(handle)?;
                let component_info = substate_ref.component_info(); // TODO: Remove clone()

                // Type check
                if !component_info.package_address.eq(&self.package_address) {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::InvalidInvocation,
                    ));
                }
                if !component_info.blueprint_name.eq(&self.blueprint_name) {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::InvalidInvocation,
                    ));
                }

                api.kernel_drop_lock(handle)?;
            }

            // Deref if global
            // TODO: Move into kernel
            let resolved_receiver =
                if let Some((derefed, derefed_lock)) = api.deref(original_node_id)? {
                    ResolvedReceiver::derefed(derefed, original_node_id, derefed_lock)
                } else {
                    ResolvedReceiver::new(original_node_id)
                };

            // Pass the component ref
            node_refs_to_copy.insert(resolved_receiver.receiver);

            (
                Some(resolved_receiver.receiver.into()),
                ResolvedActor::method(FnIdentifier::Scrypto(scrypto_fn_ident), resolved_receiver),
            )
        } else {
            (
                None,
                ResolvedActor::function(FnIdentifier::Scrypto(scrypto_fn_ident)),
            )
        };

        let handle = api.kernel_lock_substate(
            RENodeId::Global(Address::Package(self.package_address)),
            NodeModuleId::PackageTypeInfo,
            SubstateOffset::PackageTypeInfo,
            LockFlags::read_only(),
        )?;
        let substate_ref = api.kernel_get_substate_ref(handle)?;
        let type_info = substate_ref.type_info().clone();
        api.kernel_drop_lock(handle)?;

        let export_name = match type_info {
            TypeInfoSubstate::NativePackage => {
                // TODO: Do we need to check against the abi? Probably not since we should be able to verify this
                // TODO: in the native package itself.
                self.fn_name.to_string() // TODO: Clean this up
            }
            TypeInfoSubstate::WasmPackage => {
                node_refs_to_copy.insert(RENodeId::Global(Address::Component(EPOCH_MANAGER)));
                node_refs_to_copy.insert(RENodeId::Global(Address::Component(CLOCK)));
                node_refs_to_copy.insert(RENodeId::Global(Address::Resource(RADIX_TOKEN)));
                node_refs_to_copy.insert(RENodeId::Global(Address::Resource(PACKAGE_TOKEN)));
                node_refs_to_copy
                    .insert(RENodeId::Global(Address::Resource(ECDSA_SECP256K1_TOKEN)));
                node_refs_to_copy.insert(RENodeId::Global(Address::Resource(EDDSA_ED25519_TOKEN)));

                let package_global = RENodeId::Global(Address::Package(self.package_address));
                let handle = api.kernel_lock_substate(
                    package_global,
                    NodeModuleId::SELF,
                    SubstateOffset::Package(PackageOffset::Info),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.kernel_get_substate_ref(handle)?;
                let info = substate_ref.package_info(); // TODO: Remove clone()
                for dependent_resource in &info.dependent_resources {
                    node_refs_to_copy
                        .insert(RENodeId::Global(Address::Resource(*dependent_resource)));
                }

                // Find the abi
                let abi = info.blueprint_abi(&self.blueprint_name).ok_or(
                    RuntimeError::InterpreterError(InterpreterError::InvalidScryptoInvocation(
                        self.package_address,
                        self.blueprint_name.clone(),
                        self.fn_name.clone(),
                        ScryptoFnResolvingError::BlueprintNotFound,
                    )),
                )?;
                let fn_abi =
                    abi.get_fn_abi(&self.fn_name)
                        .ok_or(RuntimeError::InterpreterError(
                            InterpreterError::InvalidScryptoInvocation(
                                self.package_address,
                                self.blueprint_name.clone(),
                                self.fn_name.clone(),
                                ScryptoFnResolvingError::MethodNotFound,
                            ),
                        ))?;

                if fn_abi.mutability.is_some() != self.receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::InvalidInvocation,
                    ));
                }

                if !match_schema_with_value(&fn_abi.input, args.as_value()) {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoInvocation(
                            self.package_address,
                            self.blueprint_name.clone(),
                            self.fn_name.clone(),
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
            package_address: self.package_address,
            export_name,
            component_id: receiver,
            args: args.into(),
        };

        // TODO: remove? currently needed for `Runtime::package_address()` API.
        node_refs_to_copy.insert(RENodeId::Global(Address::Package(self.package_address)));

        Ok((
            actor,
            CallFrameUpdate {
                nodes_to_move,
                node_refs_to_copy,
            },
            executor,
        ))
    }
}

pub struct ScryptoExecutor {
    pub package_address: PackageAddress,
    pub export_name: String,
    pub component_id: Option<ComponentId>,
    pub args: ScryptoValue,
}

impl Executor for ScryptoExecutor {
    type Output = ScryptoValue;

    fn execute<Y, W>(self, api: &mut Y) -> Result<(ScryptoValue, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + KernelWasmApi<W>
            + ClientApi<RuntimeError>
            + ClientNodeApi<RuntimeError>
            + ClientSubstateApi<RuntimeError>
            + ClientSubstateApi<RuntimeError>
            + ClientPackageApi<RuntimeError>
            + ClientComponentApi<RuntimeError>
            + ClientActorApi<RuntimeError>
            + ClientUnsafeApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
        W: WasmEngine,
    {
        // Make dependent resources/components visible
        {
            let handle = api.kernel_lock_substate(
                RENodeId::Global(Address::Package(self.package_address)),
                NodeModuleId::SELF,
                SubstateOffset::Package(PackageOffset::Info),
                LockFlags::read_only(),
            )?;
            api.kernel_drop_lock(handle)?;
        }

        let handle = api.kernel_lock_substate(
            RENodeId::Global(Address::Package(self.package_address)),
            NodeModuleId::PackageTypeInfo,
            SubstateOffset::PackageTypeInfo,
            LockFlags::read_only(),
        )?;
        let substate_ref = api.kernel_get_substate_ref(handle)?;
        let type_info = substate_ref.type_info().clone();
        api.kernel_drop_lock(handle)?;

        let output = match type_info {
            TypeInfoSubstate::NativePackage => {
                let handle = api.kernel_lock_substate(
                    RENodeId::Global(Address::Package(self.package_address)),
                    NodeModuleId::SELF,
                    SubstateOffset::Package(PackageOffset::NativeCode),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.kernel_get_substate_ref(handle)?;
                let native_package_code_id = substate_ref.native_code().native_package_code_id;
                api.kernel_drop_lock(handle)?;
                NativeVm::invoke_native_package(
                    native_package_code_id,
                    self.component_id,
                    &self.export_name,
                    self.args,
                    api,
                )?
            }
            TypeInfoSubstate::WasmPackage => {
                let rtn_type = {
                    let handle = api.kernel_lock_substate(
                        RENodeId::Global(Address::Package(self.package_address)),
                        NodeModuleId::SELF,
                        SubstateOffset::Package(PackageOffset::Info),
                        LockFlags::read_only(),
                    )?;
                    let substate_ref = api.kernel_get_substate_ref(handle)?;
                    let package_info = substate_ref.package_info();
                    let fn_abi = package_info
                        .fn_abi(&self.export_name)
                        .expect("TODO: Remove this expect");
                    let rtn_type = fn_abi.output.clone();
                    api.kernel_drop_lock(handle)?;
                    rtn_type
                };

                let wasm_code = {
                    let handle = api.kernel_lock_substate(
                        RENodeId::Global(Address::Package(self.package_address)),
                        NodeModuleId::SELF,
                        SubstateOffset::Package(PackageOffset::WasmCode),
                        LockFlags::read_only(),
                    )?;
                    let substate_ref = api.kernel_get_substate_ref(handle)?;
                    let package = substate_ref.wasm_code().clone(); // TODO: Remove clone()
                    api.kernel_drop_lock(handle)?;

                    package
                };

                // Emit event
                let mut instance = api
                    .kernel_get_scrypto_interpreter()
                    .create_instance(self.package_address, &wasm_code.code);

                let output = {
                    let mut runtime: Box<dyn WasmRuntime> = Box::new(ScryptoRuntime::new(api));

                    let mut input = Vec::new();
                    if let Some(component_id) = self.component_id {
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
                                scrypto_encode(&self.args).expect("Failed to encode args"),
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

                output
            }
        };

        let update = CallFrameUpdate {
            node_refs_to_copy: output
                .global_references()
                .into_iter()
                .map(|a| RENodeId::Global(a))
                .collect(),
            nodes_to_move: output
                .owned_node_ids()
                .map_err(|e| RuntimeError::KernelError(KernelError::ReadOwnedNodesError(e)))?
                .into_iter()
                .collect(),
        };

        Ok((output.into(), update))
    }
}

struct NativeVm;

impl NativeVm {
    pub fn invoke_native_package<Y>(
        native_package_code_id: u8,
        receiver: Option<ComponentId>,
        export_name: &str,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        match native_package_code_id {
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
