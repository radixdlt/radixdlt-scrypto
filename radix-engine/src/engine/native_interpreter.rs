use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::*;
use crate::types::*;
use crate::wasm::*;

pub struct NativeInterpreter;

impl NativeInterpreter {
    pub fn run<'s, Y, W, I, R>(
        receiver: Option<Receiver>,
        auth_zone_frame_id: Option<usize>,
        fn_identifier: NativeFnIdentifier,
        input: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, RuntimeError>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        match (receiver, fn_identifier) {
            (None, NativeFnIdentifier::TransactionProcessor(transaction_processor_fn)) => {
                TransactionProcessor::static_main(transaction_processor_fn, input, system_api)
                    .map_err(|e| match e {
                        InvokeError::Downstream(runtime_error) => runtime_error,
                        InvokeError::Error(e) => RuntimeError::ApplicationError(
                            ApplicationError::TransactionProcessorError(e),
                        ),
                    })
            }
            (None, NativeFnIdentifier::Package(package_fn)) => {
                ValidatedPackage::static_main(package_fn, input, system_api).map_err(|e| match e {
                    InvokeError::Downstream(runtime_error) => runtime_error,
                    InvokeError::Error(e) => {
                        RuntimeError::ApplicationError(ApplicationError::PackageError(e))
                    }
                })
            }
            (None, NativeFnIdentifier::ResourceManager(resource_manager_fn)) => {
                ResourceManager::static_main(resource_manager_fn, input, system_api).map_err(|e| {
                    match e {
                        InvokeError::Downstream(runtime_error) => runtime_error,
                        InvokeError::Error(e) => RuntimeError::ApplicationError(
                            ApplicationError::ResourceManagerError(e),
                        ),
                    }
                })
            }
            (Some(Receiver::Consumed(node_id)), NativeFnIdentifier::Bucket(bucket_fn)) => {
                Bucket::consuming_main(node_id, bucket_fn, input, system_api).map_err(|e| match e {
                    InvokeError::Downstream(runtime_error) => runtime_error,
                    InvokeError::Error(e) => {
                        RuntimeError::ApplicationError(ApplicationError::BucketError(e))
                    }
                })
            }
            (Some(Receiver::Consumed(node_id)), NativeFnIdentifier::Proof(proof_fn)) => {
                Proof::main_consume(node_id, proof_fn, input, system_api).map_err(|e| match e {
                    InvokeError::Downstream(runtime_error) => runtime_error,
                    InvokeError::Error(e) => {
                        RuntimeError::ApplicationError(ApplicationError::ProofError(e))
                    }
                })
            }
            (Some(Receiver::CurrentAuthZone), NativeFnIdentifier::AuthZone(auth_zone_fn)) => {
                AuthZone::main(
                    auth_zone_frame_id.expect("AuthZone receiver frame id not specified"),
                    auth_zone_fn,
                    input,
                    system_api,
                )
                .map_err(|e| match e {
                    InvokeError::Downstream(runtime_error) => runtime_error,
                    InvokeError::Error(e) => {
                        RuntimeError::ApplicationError(ApplicationError::AuthZoneError(e))
                    }
                })
            }
            (
                Some(Receiver::Ref(RENodeId::Bucket(bucket_id))),
                NativeFnIdentifier::Bucket(bucket_fn),
            ) => Bucket::main(bucket_id, bucket_fn, input, system_api).map_err(|e| match e {
                InvokeError::Downstream(runtime_error) => runtime_error,
                InvokeError::Error(e) => {
                    RuntimeError::ApplicationError(ApplicationError::BucketError(e))
                }
            }),
            (
                Some(Receiver::Ref(RENodeId::Proof(proof_id))),
                NativeFnIdentifier::Proof(proof_fn),
            ) => Proof::main(proof_id, proof_fn, input, system_api).map_err(|e| match e {
                InvokeError::Downstream(runtime_error) => runtime_error,
                InvokeError::Error(e) => {
                    RuntimeError::ApplicationError(ApplicationError::ProofError(e))
                }
            }),
            (Some(Receiver::Ref(RENodeId::Worktop)), NativeFnIdentifier::Worktop(worktop_fn)) => {
                Worktop::main(worktop_fn, input, system_api).map_err(|e| match e {
                    InvokeError::Downstream(runtime_error) => runtime_error,
                    InvokeError::Error(worktop_error) => RuntimeError::ApplicationError(
                        ApplicationError::WorktopError(worktop_error),
                    ),
                })
            }
            (
                Some(Receiver::Ref(RENodeId::Vault(vault_id))),
                NativeFnIdentifier::Vault(vault_fn),
            ) => Vault::main(vault_id, vault_fn, input, system_api).map_err(|e| match e {
                InvokeError::Downstream(runtime_error) => runtime_error,
                InvokeError::Error(e) => {
                    RuntimeError::ApplicationError(ApplicationError::VaultError(e))
                }
            }),
            (
                Some(Receiver::Ref(RENodeId::Component(component_address))),
                NativeFnIdentifier::Component(component_fn),
            ) => ComponentInfo::main(component_address, component_fn, input, system_api).map_err(
                |e| match e {
                    InvokeError::Downstream(runtime_error) => runtime_error,
                    InvokeError::Error(e) => {
                        RuntimeError::ApplicationError(ApplicationError::ComponentError(e))
                    }
                },
            ),
            (
                Some(Receiver::Ref(RENodeId::ResourceManager(resource_address))),
                NativeFnIdentifier::ResourceManager(resource_manager_fn),
            ) => ResourceManager::main(resource_address, resource_manager_fn, input, system_api)
                .map_err(|e| match e {
                    InvokeError::Downstream(runtime_error) => runtime_error,
                    InvokeError::Error(e) => {
                        RuntimeError::ApplicationError(ApplicationError::ResourceManagerError(e))
                    }
                }),
            (Some(Receiver::Ref(RENodeId::System)), NativeFnIdentifier::System(system_fn)) => {
                System::main(system_fn, input, system_api).map_err(|e| match e {
                    InvokeError::Downstream(runtime_error) => runtime_error,
                    InvokeError::Error(e) => {
                        RuntimeError::ApplicationError(ApplicationError::SystemError(e))
                    }
                })
            }
            _ => {
                return Err(RuntimeError::KernelError(KernelError::MethodNotFound(
                    FnIdentifier::Native(fn_identifier.clone()),
                )))
            }
        }
    }
}
