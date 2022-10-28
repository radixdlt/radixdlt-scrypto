use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::*;
use crate::types::*;

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

impl Into<ApplicationError> for EpochManagerError {
    fn into(self) -> ApplicationError {
        ApplicationError::EpochManagerError(self)
    }
}

trait NativeFunctionActor<I, O, E> {
    fn execute<'s, Y, R>(input: I, system_api: &mut Y) -> Result<O, InvokeError<E>>
        where
            Y: SystemApi<'s, R>,
            R: FeeReserve;
}


impl NativeInterpreter {
    pub fn run_function<'s, Y, R>(
        fn_identifier: NativeFunction,
        input: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, RuntimeError>
    where
        Y: SystemApi<'s, R>,
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
            NativeFunction::EpochManager(func) => {
                EpochManager::static_main(func, input, system_api).map_err(|e| e.into())
            }
        }
    }

    pub fn run_method<'s, Y, R>(
        native_method: NativeMethod,
        resolved_receiver: ResolvedReceiver,
        input: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, RuntimeError>
    where
        Y: SystemApi<'s, R>,
        R: FeeReserve,
    {
        match (resolved_receiver.receiver, native_method.clone()) {
            (RENodeId::AuthZoneStack(auth_zone_id), NativeMethod::AuthZone(method)) => {
                AuthZoneStack::main(auth_zone_id, method, input, system_api).map_err(|e| e.into())
            }
            (RENodeId::Bucket(bucket_id), NativeMethod::Bucket(method)) => {
                Bucket::main(bucket_id, method, input, system_api).map_err(|e| e.into())
            }
            (RENodeId::Proof(proof_id), NativeMethod::Proof(method)) => {
                Proof::main(proof_id, method, input, system_api).map_err(|e| e.into())
            }
            (RENodeId::Worktop, NativeMethod::Worktop(method)) => {
                Worktop::main(method, input, system_api).map_err(|e| e.into())
            }
            (RENodeId::Vault(vault_id), NativeMethod::Vault(method)) => {
                Vault::main(vault_id, method, input, system_api).map_err(|e| e.into())
            }
            (RENodeId::Component(component_id), NativeMethod::Component(method)) => {
                Component::main(component_id, method, input, system_api).map_err(|e| e.into())
            }
            (
                RENodeId::ResourceManager(resource_address),
                NativeMethod::ResourceManager(method),
            ) => ResourceManager::main(resource_address, method, input, system_api)
                .map_err(|e| e.into()),
            (RENodeId::EpochManager(component_id), NativeMethod::EpochManager(method)) => {
                EpochManager::main(component_id, method, input, system_api).map_err(|e| e.into())
            }
            (receiver, _) => {
                return Err(RuntimeError::KernelError(
                    KernelError::MethodReceiverNotMatch(native_method, receiver),
                ));
            }
        }
    }
}
