use crate::blueprints::transaction_processor::TransactionProcessorError;
use crate::errors::{ApplicationError, ScryptoFnResolvingError};
use crate::errors::{InterpreterError, KernelError, RuntimeError};
use crate::kernel::kernel_api::{KernelSubstateApi, KernelWasmApi, LockFlags};
use crate::kernel::*;
use crate::types::*;
use crate::wasm::{WasmEngine, WasmInstance, WasmInstrumenter, WasmMeteringConfig, WasmRuntime};
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::api::types::{ScryptoInvocation, ScryptoReceiver};
use radix_engine_interface::api::{ClientActorApi, ClientApi, ClientComponentApi, ClientMeteringApi, ClientNodeApi, ClientStaticInvokeApi, ClientSubstateApi};
use radix_engine_interface::api::{ClientDerefApi, ClientPackageApi};
use radix_engine_interface::blueprints::access_controller::AccessControllerCreateGlobalInvocation;
use radix_engine_interface::blueprints::account::{AccountCreateInvocation, AccountNewInvocation};
use radix_engine_interface::blueprints::clock::ClockCreateInvocation;
use radix_engine_interface::blueprints::epoch_manager::EpochManagerCreateInvocation;
use radix_engine_interface::data::*;
use radix_engine_interface::data::{match_schema_with_value, ScryptoValue};
use crate::blueprints::identity::IdentityCreateExecutable;

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
                ScryptoReceiver::Global(component_address) => match component_address {
                    ComponentAddress::Normal(..)
                    | ComponentAddress::Account(..)
                    | ComponentAddress::EcdsaSecp256k1VirtualAccount(..)
                    | ComponentAddress::EddsaEd25519VirtualAccount(..) => {
                        RENodeId::Global(GlobalAddress::Component(component_address))
                    }
                    ComponentAddress::Clock(..)
                    | ComponentAddress::EpochManager(..)
                    | ComponentAddress::Validator(..)
                    | ComponentAddress::Identity(..)
                    | ComponentAddress::AccessController(..)
                    | ComponentAddress::EcdsaSecp256k1VirtualIdentity(..)
                    | ComponentAddress::EddsaEd25519VirtualIdentity(..) => {
                        return Err(RuntimeError::InterpreterError(
                            InterpreterError::InvalidInvocation,
                        ));
                    }
                },
                ScryptoReceiver::Component(component_id) => RENodeId::Component(component_id),
            };

            // Type Check
            {
                let handle = api.lock_substate(
                    original_node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Component(ComponentOffset::Info),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.get_ref(handle)?;
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

                api.drop_lock(handle)?;
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


        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Resource(RADIX_TOKEN)));

        match self.package_address {
            IDENTITY_PACKAGE | EPOCH_MANAGER_PACKAGE | CLOCK_PACKAGE | ACCOUNT_PACKAGE | ACCESS_CONTROLLER_PACKAGE => {
                let executor = ScryptoExecutor {
                    package_address: self.package_address,
                    export_name: "test".to_string(),
                    component_id: receiver,
                    args: args.into(),
                };

                return Ok((
                    actor,
                    CallFrameUpdate {
                        nodes_to_move,
                        node_refs_to_copy,
                    },
                    executor,
                ))
            }
            _ => {
            }
        }


        // Signature check + retrieve export_name
        let export_name = {
            let package_global = RENodeId::Global(GlobalAddress::Package(self.package_address));

            let handle = api.lock_substate(
                package_global,
                NodeModuleId::SELF,
                SubstateOffset::Package(PackageOffset::Info),
                LockFlags::read_only(),
            )?;
            let substate_ref = api.get_ref(handle)?;
            let package = substate_ref.package_info(); // TODO: Remove clone()
                                                       // Find the abi
            let abi = package.blueprint_abi(&self.blueprint_name).ok_or(
                RuntimeError::InterpreterError(InterpreterError::InvalidScryptoInvocation(
                    self.package_address,
                    self.blueprint_name.clone(),
                    self.fn_name.clone(),
                    ScryptoFnResolvingError::BlueprintNotFound,
                )),
            )?;
            let fn_abi = abi
                .get_fn_abi(&self.fn_name)
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
            api.drop_lock(handle)?;

            export_name
        };

        let executor = ScryptoExecutor {
            package_address: self.package_address,
            export_name,
            component_id: receiver,
            args: args.into(),
        };

        // TODO: remove? currently needed for `Runtime::package_address()` API.
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Package(
            self.package_address,
        )));
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Component(EPOCH_MANAGER)));
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Component(CLOCK)));
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Resource(
            ECDSA_SECP256K1_TOKEN,
        )));
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Resource(
            EDDSA_ED25519_TOKEN,
        )));

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
            + ClientMeteringApi<RuntimeError>
            + ClientStaticInvokeApi<RuntimeError>,
        W: WasmEngine,
    {
        match self.package_address {
            IDENTITY_PACKAGE => {
                let invocation: IdentityCreateExecutable = scrypto_decode(&scrypto_encode(&self.args).unwrap()).unwrap();
                let rtn = invocation.execute(api)?;
                return Ok((scrypto_decode(&scrypto_encode(&rtn.0).unwrap()).unwrap(), rtn.1));
            }
            EPOCH_MANAGER_PACKAGE => {
                let invocation: EpochManagerCreateInvocation = scrypto_decode(&scrypto_encode(&self.args).unwrap()).unwrap();
                let rtn = invocation.execute(api)?;
                return Ok((scrypto_decode(&scrypto_encode(&rtn.0).unwrap()).unwrap(), rtn.1));
            }
            CLOCK_PACKAGE => {
                let invocation: ClockCreateInvocation = scrypto_decode(&scrypto_encode(&self.args).unwrap()).unwrap();
                let rtn = invocation.execute(api)?;
                return Ok((scrypto_decode(&scrypto_encode(&rtn.0).unwrap()).unwrap(), rtn.1));
            }
            ACCOUNT_PACKAGE => {
                // TODO: Add Account Create
                let invocation: AccountNewInvocation = scrypto_decode(&scrypto_encode(&self.args).unwrap()).unwrap();
                let rtn = invocation.execute(api)?;
                return Ok((scrypto_decode(&scrypto_encode(&rtn.0).unwrap()).unwrap(), rtn.1));
            }
            ACCESS_CONTROLLER_PACKAGE => {
                let invocation: AccessControllerCreateGlobalInvocation = scrypto_decode(&scrypto_encode(&self.args).unwrap()).unwrap();
                let rtn = invocation.execute(api)?;
                return Ok((scrypto_decode(&scrypto_encode(&rtn.0).unwrap()).unwrap(), rtn.1));
            }
            _ => {
            }
        }

        let package = {
            let handle = api.lock_substate(
                RENodeId::Global(GlobalAddress::Package(self.package_address)),
                NodeModuleId::SELF,
                SubstateOffset::Package(PackageOffset::Info),
                LockFlags::read_only(),
            )?;
            let substate_ref = api.get_ref(handle)?;
            let package = substate_ref.package_info().clone(); // TODO: Remove clone()
            api.drop_lock(handle)?;

            package
        };

        let fn_abi = package
            .fn_abi(&self.export_name)
            .expect("TODO: Remove this expect");
        let rtn_type = fn_abi.output.clone();

        // Emit event
        api.emit_wasm_instantiation_event(package.code())?;
        let mut instance = api
            .scrypto_interpreter()
            .create_instance(self.package_address, &package.code);

        let output = {
            let mut runtime: Box<dyn WasmRuntime> = Box::new(ScryptoRuntime::new(api));

            let mut input = Vec::new();
            if let Some(component_id) = self.component_id {
                input.push(
                    runtime
                        .allocate_buffer(
                            scrypto_encode(&component_id).expect("Failed to encode component id"),
                        )
                        .expect("Failed to allocate buffer"),
                );
            }
            input.push(
                runtime
                    .allocate_buffer(scrypto_encode(&self.args).expect("Failed to encode args"))
                    .expect("Failed to allocate buffer"),
            );

            instance.invoke_export(&self.export_name, input, &mut runtime)?
        };
        let output = IndexedScryptoValue::from_vec(output).map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::InvalidScryptoReturn(e))
        })?;

        let rtn = if !match_schema_with_value(&rtn_type, output.as_value()) {
            Err(RuntimeError::KernelError(
                KernelError::InvalidScryptoFnOutput,
            ))
        } else {
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
        };

        rtn
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
            assert_sync::<crate::kernel::ScryptoInterpreter<crate::wasm::DefaultWasmEngine>>();
        }
    };
}
