use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::*;
use crate::types::*;
use crate::wasm::*;
use scrypto::core::{FnIdent, MethodFnIdent, MethodIdent, NativeFunctionFnIdent};

pub struct NativeInterpreter;

impl<E: Into<ApplicationError>> Into<RuntimeError> for InvokeError<E> {
    fn into(self) -> RuntimeError {
        match self {
            InvokeError::Downstream(runtime_error) => runtime_error,
            InvokeError::Error(e) => RuntimeError::ApplicationError(e.into()),
        }
    }
}

impl Into<ApplicationError> for TransactionProcessorError {
    fn into(self) -> ApplicationError {
        ApplicationError::TransactionProcessorError(self)
    }
}

impl Into<ApplicationError> for PackageError {
    fn into(self) -> ApplicationError {
        ApplicationError::PackageError(self)
    }
}

impl Into<ApplicationError> for ResourceManagerError {
    fn into(self) -> ApplicationError {
        ApplicationError::ResourceManagerError(self)
    }
}

impl Into<ApplicationError> for BucketError {
    fn into(self) -> ApplicationError {
        ApplicationError::BucketError(self)
    }
}

impl Into<ApplicationError> for ProofError {
    fn into(self) -> ApplicationError {
        ApplicationError::ProofError(self)
    }
}

impl Into<ApplicationError> for AuthZoneError {
    fn into(self) -> ApplicationError {
        ApplicationError::AuthZoneError(self)
    }
}

impl Into<ApplicationError> for WorktopError {
    fn into(self) -> ApplicationError {
        ApplicationError::WorktopError(self)
    }
}

impl Into<ApplicationError> for VaultError {
    fn into(self) -> ApplicationError {
        ApplicationError::VaultError(self)
    }
}

impl Into<ApplicationError> for ComponentError {
    fn into(self) -> ApplicationError {
        ApplicationError::ComponentError(self)
    }
}

impl Into<ApplicationError> for SystemError {
    fn into(self) -> ApplicationError {
        ApplicationError::SystemError(self)
    }
}

impl NativeInterpreter {
    pub fn run_function<'s, Y, W, I, R>(
        fn_identifier: NativeFunctionFnIdent,
        input: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, RuntimeError>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        match fn_identifier {
            NativeFunctionFnIdent::TransactionProcessor(transaction_processor_fn) => {
                TransactionProcessor::static_main(transaction_processor_fn, input, system_api)
                    .map_err(|e| e.into())
            }
            NativeFunctionFnIdent::Package(package_fn) => {
                Package::static_main(package_fn, input, system_api).map_err(|e| e.into())
            }
            NativeFunctionFnIdent::ResourceManager(resource_manager_fn) => {
                ResourceManager::static_main(resource_manager_fn, input, system_api)
                    .map_err(|e| e.into())
            }
            NativeFunctionFnIdent::System(system_fn) => {
                System::static_main(system_fn, input, system_api).map_err(|e| e.into())
            }
        }
    }

    pub fn run_method<'s, Y, W, I, R>(
        receiver: Receiver,
        fn_identifier: NativeMethodFnIdent,
        input: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, RuntimeError>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        match (receiver.clone(), fn_identifier.clone()) {
            (Receiver::Consumed(node_id), NativeMethodFnIdent::Bucket(bucket_fn)) => {
                Bucket::consuming_main(node_id, bucket_fn, input, system_api).map_err(|e| e.into())
            }
            (Receiver::Consumed(node_id), NativeMethodFnIdent::Proof(proof_fn)) => {
                Proof::main_consume(node_id, proof_fn, input, system_api).map_err(|e| e.into())
            }
            (
                Receiver::Ref(RENodeId::AuthZone(auth_zone_id)),
                NativeMethodFnIdent::AuthZone(auth_zone_fn),
            ) => {
                AuthZone::main(auth_zone_id, auth_zone_fn, input, system_api).map_err(|e| e.into())
            }
            (
                Receiver::Ref(RENodeId::Bucket(bucket_id)),
                NativeMethodFnIdent::Bucket(bucket_fn),
            ) => Bucket::main(bucket_id, bucket_fn, input, system_api).map_err(|e| e.into()),
            (Receiver::Ref(RENodeId::Proof(proof_id)), NativeMethodFnIdent::Proof(proof_fn)) => {
                Proof::main(proof_id, proof_fn, input, system_api).map_err(|e| e.into())
            }
            (Receiver::Ref(RENodeId::Worktop), NativeMethodFnIdent::Worktop(worktop_fn)) => {
                Worktop::main(worktop_fn, input, system_api).map_err(|e| e.into())
            }
            (Receiver::Ref(RENodeId::Vault(vault_id)), NativeMethodFnIdent::Vault(vault_fn)) => {
                Vault::main(vault_id, vault_fn, input, system_api).map_err(|e| e.into())
            }
            (
                Receiver::Ref(RENodeId::Component(component_address)),
                NativeMethodFnIdent::Component(component_fn),
            ) => Component::main(component_address, component_fn, input, system_api)
                .map_err(|e| e.into()),
            (
                Receiver::Ref(RENodeId::ResourceManager(resource_address)),
                NativeMethodFnIdent::ResourceManager(resource_manager_fn),
            ) => ResourceManager::main(resource_address, resource_manager_fn, input, system_api)
                .map_err(|e| e.into()),
            (
                Receiver::Ref(RENodeId::System(component_address)),
                NativeMethodFnIdent::System(system_fn),
            ) => {
                System::main(component_address, system_fn, input, system_api).map_err(|e| e.into())
            }
            _ => {
                return Err(RuntimeError::KernelError(KernelError::FnIdentNotFound(
                    FnIdent::Method(MethodIdent {
                        receiver,
                        fn_ident: MethodFnIdent::Native(fn_identifier),
                    }),
                )))
            }
        }
    }
}
