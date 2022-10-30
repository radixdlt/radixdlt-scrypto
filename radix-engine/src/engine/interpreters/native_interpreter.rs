use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::*;
use crate::types::*;
use sbor::rust::fmt::Debug;
use sbor::*;

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

pub trait InvokableNativeFunction<'a>:
    Invokable<EpochManagerCreateInput>
    + Invokable<PackagePublishInput>
    + Invokable<ResourceManagerBurnInput>
    + Invokable<ResourceManagerCreateInput>
    + Invokable<TransactionProcessorRunInput<'a>>
{
}

impl<N: NativeExecutable> Invocation for N {
    type Output = <N as NativeExecutable>::Output;
}

pub struct NativeResolver;

impl<N: NativeFunctionInvocation> Resolver<N> for NativeResolver {
    type Exec = NativeExecutor<N>;

    fn resolve<D: MethodDeref>(
        invocation: N,
        _deref: &D,
    ) -> (REActor, CallFrameUpdate, Self::Exec) {
        let native_function = N::native_function();
        let call_frame_update = invocation.call_frame_update();
        let actor = REActor::Function(ResolvedFunction::Native(native_function));
        let input = ScryptoValue::from_typed(&invocation);
        let executor = NativeExecutor(invocation, input);
        (actor, call_frame_update, executor)
    }
}

pub trait NativeExecutable: Invocation {
    type Output: Debug;

    fn execute<'s, 'a, Y, R>(
        invocation: Self,
        system_api: &mut Y,
    ) -> Result<(<Self as Invocation>::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R>
            + Invokable<ScryptoInvocation>
            + InvokableNativeFunction<'a>
            + Invokable<NativeMethodInvocation>,
        R: FeeReserve;
}

pub trait NativeFunctionInvocation: NativeExecutable + Encode + Debug {
    fn native_function() -> NativeFunction;

    fn call_frame_update(&self) -> CallFrameUpdate;
}

pub struct NativeExecutor<N: NativeExecutable>(pub N, pub ScryptoValue);

impl<N: NativeExecutable> Executor for NativeExecutor<N> {
    type Output = <N as Invocation>::Output;

    fn args(&self) -> &ScryptoValue {
        &self.1
    }

    fn execute<'s, 'a, Y, R>(
        self,
        system_api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R>
            + Invokable<ScryptoInvocation>
            + InvokableNativeFunction<'a>
            + Invokable<NativeMethodInvocation>,
        R: FeeReserve,
    {
        N::execute(self.0, system_api)
    }
}

pub struct NativeMethodExecutor(pub NativeMethod, pub ResolvedReceiver, pub ScryptoValue);

impl Executor for NativeMethodExecutor {
    type Output = ScryptoValue;

    fn args(&self) -> &ScryptoValue {
        &self.2
    }

    fn execute<'s, 'a, Y, R>(
        self,
        system_api: &mut Y,
    ) -> Result<(ScryptoValue, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R>
            + Invokable<ScryptoInvocation>
            + InvokableNativeFunction<'a>
            + Invokable<NativeMethodInvocation>,
        R: FeeReserve,
    {
        let output = match (self.1.receiver, self.0) {
            (RENodeId::AuthZoneStack(auth_zone_id), NativeMethod::AuthZone(method)) => {
                AuthZoneStack::main(auth_zone_id, method, self.2, system_api)
                    .map_err::<RuntimeError, _>(|e| e.into())
            }
            (RENodeId::Bucket(bucket_id), NativeMethod::Bucket(method)) => {
                Bucket::main(bucket_id, method, self.2, system_api)
                    .map_err::<RuntimeError, _>(|e| e.into())
            }
            (RENodeId::Proof(proof_id), NativeMethod::Proof(method)) => {
                Proof::main(proof_id, method, self.2, system_api)
                    .map_err::<RuntimeError, _>(|e| e.into())
            }
            (RENodeId::Worktop, NativeMethod::Worktop(method)) => {
                Worktop::main(method, self.2, system_api).map_err::<RuntimeError, _>(|e| e.into())
            }
            (RENodeId::Vault(vault_id), NativeMethod::Vault(method)) => {
                Vault::main(vault_id, method, self.2, system_api)
                    .map_err::<RuntimeError, _>(|e| e.into())
            }
            (RENodeId::Component(component_id), NativeMethod::Component(method)) => {
                Component::main(component_id, method, self.2, system_api)
                    .map_err::<RuntimeError, _>(|e| e.into())
            }
            (
                RENodeId::ResourceManager(resource_address),
                NativeMethod::ResourceManager(method),
            ) => ResourceManager::main(resource_address, method, self.2, system_api)
                .map_err::<RuntimeError, _>(|e| e.into()),
            (RENodeId::EpochManager(component_id), NativeMethod::EpochManager(method)) => {
                EpochManager::main(component_id, method, self.2, system_api)
                    .map_err::<RuntimeError, _>(|e| e.into())
            }
            (receiver, native_method) => {
                return Err(RuntimeError::KernelError(
                    KernelError::MethodReceiverNotMatch(native_method, receiver),
                ));
            }
        }?;

        let update = CallFrameUpdate {
            node_refs_to_copy: output
                .global_references()
                .into_iter()
                .map(|a| RENodeId::Global(a))
                .collect(),
            nodes_to_move: output.node_ids().into_iter().collect(),
        };

        Ok((output, update))
    }
}
