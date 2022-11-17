use crate::engine::*;
use crate::model::*;
use crate::types::*;
use radix_engine_lib::data::{IndexedScryptoValue, ScryptoCustomTypeId};
use radix_engine_lib::engine::api::{SysInvokableNative, EngineApi};
use radix_engine_lib::engine::types::{NativeFunction, NativeMethod, RENodeId};
use radix_engine_lib::model::*;
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

// TODO: This should be cleaned up
#[derive(Debug)]
pub enum NativeInvocationInfo {
    Function(NativeFunction, CallFrameUpdate),
    Method(NativeMethod, RENodeId, CallFrameUpdate),
}

impl<N: NativeExecutable> Invocation for N {
    type Output = <N as NativeExecutable>::NativeOutput;
}

pub struct NativeResolver;

impl<N: NativeInvocation> Resolver<N> for NativeResolver {
    type Exec = NativeExecutor<N>;

    fn resolve<D: MethodDeref>(
        invocation: N,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let info = invocation.info();
        let (actor, call_frame_update) = match info {
            NativeInvocationInfo::Method(method, receiver, mut call_frame_update) => {
                // TODO: Move this logic into kernel
                let resolved_receiver = if let Some(derefed) = deref.deref(receiver)? {
                    ResolvedReceiver::derefed(derefed, receiver)
                } else {
                    ResolvedReceiver::new(receiver)
                };
                let resolved_node_id = resolved_receiver.receiver;
                call_frame_update.node_refs_to_copy.insert(resolved_node_id);

                let actor = REActor::Method(ResolvedMethod::Native(method), resolved_receiver);
                (actor, call_frame_update)
            }
            NativeInvocationInfo::Function(native_function, call_frame_update) => {
                let actor = REActor::Function(ResolvedFunction::Native(native_function));
                (actor, call_frame_update)
            }
        };

        let input = IndexedScryptoValue::from_typed(&invocation);
        let executor = NativeExecutor(invocation, input);
        Ok((actor, call_frame_update, executor))
    }
}

pub trait NativeInvocation: NativeExecutable + Encode<ScryptoCustomTypeId> + Debug {
    fn info(&self) -> NativeInvocationInfo;
}

pub trait NativeExecutable: Invocation {
    type NativeOutput: Traceable + 'static;

    fn execute<Y>(
        invocation: Self,
        system_api: &mut Y,
    ) -> Result<(<Self as Invocation>::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi
            + Invokable<ScryptoInvocation>
            + EngineApi<RuntimeError>
            + SysInvokableNative<RuntimeError>
            + Invokable<ResourceManagerSetResourceAddressInvocation>;
}

pub struct NativeExecutor<N: NativeExecutable>(pub N, pub IndexedScryptoValue);

impl<N: NativeExecutable> Executor for NativeExecutor<N> {
    type Output = <N as Invocation>::Output;

    fn args(&self) -> &IndexedScryptoValue {
        &self.1
    }

    fn execute<Y>(self, system_api: &mut Y) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi
            + Invokable<ScryptoInvocation>
            + EngineApi<RuntimeError>
            + SysInvokableNative<RuntimeError>
            + Invokable<ResourceManagerSetResourceAddressInvocation>,
    {
        N::execute(self.0, system_api)
    }
}
