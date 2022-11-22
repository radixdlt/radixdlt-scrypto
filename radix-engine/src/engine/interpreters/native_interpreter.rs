use crate::engine::*;
use crate::model::*;
use crate::types::*;
use radix_engine_interface::api::api::{EngineApi, Invocation, SysInvokableNative};
use radix_engine_interface::api::types::{NativeFunction, NativeMethod, RENodeId};
use radix_engine_interface::data::{IndexedScryptoValue, ScryptoCustomTypeId};
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

impl Into<ApplicationError> for AccessRulesError {
    fn into(self) -> ApplicationError {
        ApplicationError::AccessRulesError(self)
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

pub trait NativeInvocation: Invocation + Encode<ScryptoCustomTypeId> + Debug {
    fn info(&self) -> NativeInvocationInfo;

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

impl<I: NativeInvocation> ExecutableInvocation for I {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError>
        where
            Self: Sized,
    {
        let input = IndexedScryptoValue::from_typed(&self);
        let info = self.info();
        let (actor, call_frame_update) = match info {
            NativeInvocationInfo::Method(method, receiver, mut call_frame_update) => {
                // TODO: Move this logic into kernel
                let resolved_receiver =
                    if let Some((derefed, derefed_lock)) = deref.deref(receiver)? {
                        ResolvedReceiver::derefed(derefed, receiver, derefed_lock)
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

        let executor = NativeExecutor(self, input);
        Ok((actor, call_frame_update, executor))
    }
}


pub struct NativeExecutor<N: NativeInvocation>(pub N, pub IndexedScryptoValue);

impl<N: NativeInvocation> Executor for NativeExecutor<N> {
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

/*
pub struct NativeMethodResolver;

impl<N: NativeInvocationMethod> Resolver<N> for NativeMethodResolver {
    type Exec = NativeMethodExecutor<N>;

    fn resolve<D: MethodDeref>(
        invocation: N,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let input = IndexedScryptoValue::from_typed(&invocation);

        let (receiver, args, method, mut call_frame_update) = invocation.resolve();
        let receiver = receiver.into();

        // TODO: Move this logic into kernel
        let resolved_receiver = if let Some((derefed, derefed_lock)) = deref.deref(receiver)? {
            ResolvedReceiver::derefed(derefed, receiver, derefed_lock)
        } else {
            ResolvedReceiver::new(receiver)
        };
        let resolved_node_id = resolved_receiver.receiver;
        call_frame_update.node_refs_to_copy.insert(resolved_node_id);

        let actor = REActor::Method(ResolvedMethod::Native(method), resolved_receiver);

        let executor = NativeMethodExecutor(receiver.into(), args, input);

        Ok((actor, call_frame_update, executor))
    }
}

pub struct NativeMethodExecutor<N: NativeInvocationMethod>(
    pub RENodeId,
    pub N::Args,
    pub IndexedScryptoValue,
);

impl<N: NativeInvocationMethod> Executor for NativeMethodExecutor<N> {
    type Output = N::Output;

    fn args(&self) -> &IndexedScryptoValue {
        &self.2
    }

    fn execute<Y>(self, system_api: &mut Y) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi
            + Invokable<ScryptoInvocation>
            + EngineApi<RuntimeError>
            + SysInvokableNative<RuntimeError>
            + Invokable<ResourceManagerSetResourceAddressInvocation>,
    {
        N::execute(self.0, self.1, system_api)
    }
}

pub trait NativeInvocationMethod: Invocation + Encode<ScryptoCustomTypeId> + Debug {
    type Receiver: Into<RENodeId>;
    type Args;

    fn resolve(self) -> (Self::Receiver, Self::Args, NativeMethod, CallFrameUpdate);

    fn execute<Y>(
        receiver: RENodeId,
        args: Self::Args,
        system_api: &mut Y,
    ) -> Result<(<Self as Invocation>::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi
            + Invokable<ScryptoInvocation>
            + EngineApi<RuntimeError>
            + SysInvokableNative<RuntimeError>
            + Invokable<ResourceManagerSetResourceAddressInvocation>;
}

 */
