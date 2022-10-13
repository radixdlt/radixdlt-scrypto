use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::*;
use crate::types::*;
use crate::wasm::*;
use scrypto::core::{FnIdent, MethodIdent, NativeFunction, ReceiverMethodIdent};

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
        fn_identifier: NativeFunction,
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
            NativeFunction::TransactionProcessor(func) => {
                TransactionProcessor::static_main(func, input, system_api).map_err(|e| e.into())
            }
            NativeFunction::Package(func) => {
                Package::static_main(func, input, system_api).map_err(|e| e.into())
            }
            NativeFunction::ResourceManager(func) => {
                ResourceManager::static_main(func, input, system_api).map_err(|e| e.into())
            }
            NativeFunction::System(func) => {
                System::static_main(func, input, system_api).map_err(|e| e.into())
            }
        }
    }

    pub fn run_method<'s, Y, W, I, R>(
        receiver: Receiver,
        native_method: NativeMethod,
        input: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, RuntimeError>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        match (receiver.clone(), native_method.clone()) {
            (Receiver::Consumed(node_id), NativeMethod::Bucket(method)) => {
                BucketSubstate::consuming_main(node_id, method, input, system_api)
                    .map_err(|e| e.into())
            }
            (Receiver::Consumed(node_id), NativeMethod::Proof(method)) => {
                ProofSubstate::main_consume(node_id, method, input, system_api)
                    .map_err(|e| e.into())
            }
            (Receiver::Ref(RENodeId::AuthZone(auth_zone_id)), NativeMethod::AuthZone(method)) => {
                AuthZoneSubstate::main(auth_zone_id, method, input, system_api)
                    .map_err(|e| e.into())
            }
            (Receiver::Ref(RENodeId::Bucket(bucket_id)), NativeMethod::Bucket(method)) => {
                BucketSubstate::main(bucket_id, method, input, system_api).map_err(|e| e.into())
            }
            (Receiver::Ref(RENodeId::Proof(proof_id)), NativeMethod::Proof(method)) => {
                ProofSubstate::main(proof_id, method, input, system_api).map_err(|e| e.into())
            }
            (Receiver::Ref(RENodeId::Worktop), NativeMethod::Worktop(method)) => {
                WorktopSubstate::main(method, input, system_api).map_err(|e| e.into())
            }
            (Receiver::Ref(RENodeId::Vault(vault_id)), NativeMethod::Vault(method)) => {
                Vault::main(vault_id, method, input, system_api).map_err(|e| e.into())
            }
            (Receiver::Ref(RENodeId::Component(component_id)), NativeMethod::Component(method)) => {
                Component::main(component_id, method, input, system_api).map_err(|e| e.into())
            }
            (
                Receiver::Ref(RENodeId::ResourceManager(resource_address)),
                NativeMethod::ResourceManager(method),
            ) => ResourceManager::main(resource_address, method, input, system_api)
                .map_err(|e| e.into()),
            (Receiver::Ref(RENodeId::System(component_id)), NativeMethod::System(method)) => {
                System::main(component_id, method, input, system_api).map_err(|e| e.into())
            }
            _ => {
                return Err(RuntimeError::KernelError(KernelError::FnIdentNotFound(
                    FnIdent::Method(ReceiverMethodIdent {
                        receiver,
                        method_ident: MethodIdent::Native(native_method),
                    }),
                )))
            }
        }
    }
}
