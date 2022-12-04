use crate::engine::*;
use crate::model::*;
use crate::types::*;
use radix_engine_interface::api::api::{EngineApi, SysInvokableNative};
use radix_engine_interface::api::types::RENodeId;
use sbor::rust::fmt::Debug;
use crate::wasm::WasmEngine;

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

impl Into<ApplicationError> for AccessRulesChainError {
    fn into(self) -> ApplicationError {
        ApplicationError::AccessRulesChainError(self)
    }
}

impl Into<ApplicationError> for EpochManagerError {
    fn into(self) -> ApplicationError {
        ApplicationError::EpochManagerError(self)
    }
}

pub trait NativeProcedure {
    type Output: Debug;
    fn main<Y>(self, system_api: &mut Y) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi
            + Invokable<ScryptoInvocation>
            + EngineApi<RuntimeError>
            + SysInvokableNative<RuntimeError>;
}

pub struct NativeExecutor<N: NativeProcedure>(pub N);

impl<N: NativeProcedure> Executor for NativeExecutor<N> {
    type Output = N::Output;

    fn execute<Y>(self, system_api: &mut Y) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi
            + Invokable<ScryptoInvocation>
            + EngineApi<RuntimeError>
            + SysInvokableNative<RuntimeError>,
    {
        self.0.main(system_api)
    }
}

pub fn deref_and_update<D: ResolveApi<W>, W: WasmEngine>(
    receiver: RENodeId,
    call_frame_update: &mut CallFrameUpdate,
    deref: &mut D,
) -> Result<ResolvedReceiver, RuntimeError> {
    // TODO: Move this logic into kernel
    let resolved_receiver = if let Some((derefed, derefed_lock)) = deref.deref(receiver)? {
        ResolvedReceiver::derefed(derefed, receiver, derefed_lock)
    } else {
        ResolvedReceiver::new(receiver)
    };
    let resolved_node_id = resolved_receiver.receiver;
    call_frame_update.node_refs_to_copy.insert(resolved_node_id);

    Ok(resolved_receiver)
}
