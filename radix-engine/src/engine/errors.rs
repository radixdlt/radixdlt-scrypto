use sbor::*;
use transaction::errors::*;

use crate::engine::{ExecutionMode, LockFlags, REActor};
use crate::model::*;
use crate::types::*;
use crate::wasm::WasmError;

use super::AuthError;
use super::CostingError;
use super::ExecutionTraceError;
use super::TrackError;

/// Represents an error which causes a tranasction to be rejected.
#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum RejectionError {
    SuccessButFeeLoanNotRepaid,
    ErrorBeforeFeeLoanRepaid(RuntimeError),
}

impl fmt::Display for RejectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Represents an error when executing a transaction.
#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum RuntimeError {
    /// An error occurred within the kernel.
    KernelError(KernelError),

    /// An error occurred within call frame.
    CallFrameError(CallFrameError),

    /// An error occurred within an interpreter
    InterpreterError(InterpreterError),

    /// An error occurred within a kernel module.
    ModuleError(ModuleError),

    /// An error occurred within application logic, like the RE models.
    ApplicationError(ApplicationError),
}

impl From<KernelError> for RuntimeError {
    fn from(error: KernelError) -> Self {
        RuntimeError::KernelError(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
pub enum KernelError {
    InvalidModeTransition(ExecutionMode, ExecutionMode),

    // invocation
    WasmError(WasmError),

    InvalidReferencePass(GlobalAddress),
    InvalidReferenceReturn(GlobalAddress),
    InvalidReferenceWrite(GlobalAddress),
    GlobalAddressNotFound(GlobalAddress),

    MaxCallDepthLimitReached,
    InvalidScryptoFnOutput,
    MethodReceiverNotMatch(NativeMethod, Receiver),

    // ID allocation
    IdAllocationError(IdAllocationError),

    // SBOR decoding
    DecodeError(DecodeError),

    // RENode
    StoredNodeRemoved(RENodeId),
    RENodeGlobalizeTypeNotAllowed(RENodeId),
    TrackError(TrackError),
    LockDoesNotExist(LockHandle),
    LockNotMutable(LockHandle),
    BlobNotFound(Hash),
    DropNodeFailure(RENodeId),

    // Substate Constraints
    InvalidOffset(SubstateOffset),
    InvalidOwnership(SubstateOffset, RENodeId),
    InvalidOverwrite,

    // Actor Constraints
    InvalidDropNodeVisibility {
        mode: ExecutionMode,
        actor: REActor,
        node_id: RENodeId,
    },
    InvalidCreateNodeVisibility {
        mode: ExecutionMode,
        actor: REActor,
    },
    InvalidSubstateVisibility {
        mode: ExecutionMode,
        actor: REActor,
        node_id: RENodeId,
        offset: SubstateOffset,
        flags: LockFlags,
    },
    CantMoveDownstream(RENodeId),
    CantMoveUpstream(RENodeId),
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
pub enum CallFrameError {
    OffsetDoesNotExist(RENodeId, SubstateOffset),
    RENodeNotVisible(RENodeId),
    RENodeNotOwned(RENodeId),
    MovingLockedRENode(RENodeId),
}

impl From<CallFrameError> for RuntimeError {
    fn from(error: CallFrameError) -> Self {
        RuntimeError::CallFrameError(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
pub enum ScryptoFnResolvingError {
    BlueprintNotFound,
    FunctionNotFound,
    MethodNotFound,
    InvalidInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
pub enum InterpreterError {
    InvalidScryptoFunctionInvocation(ScryptoFunctionIdent, ScryptoFnResolvingError),
    InvalidScryptoMethodInvocation(ScryptoMethodIdent, ScryptoFnResolvingError),
    InvalidNativeFunctionIdent(NativeFunctionIdent),
    InvalidNativeMethodIdent(NativeMethodIdent),
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
pub enum ModuleError {
    AuthError(AuthError),
    CostingError(CostingError),
    ExecutionTraceError(ExecutionTraceError),
}

impl Into<ModuleError> for AuthError {
    fn into(self) -> ModuleError {
        ModuleError::AuthError(self)
    }
}

#[derive(Debug)]
pub enum InvokeError<E> {
    Error(E),
    Downstream(RuntimeError),
}

impl<E> From<RuntimeError> for InvokeError<E> {
    fn from(runtime_error: RuntimeError) -> Self {
        InvokeError::Downstream(runtime_error)
    }
}

impl<E> InvokeError<E> {
    pub fn error(error: E) -> Self {
        InvokeError::Error(error)
    }

    pub fn downstream(runtime_error: RuntimeError) -> Self {
        InvokeError::Downstream(runtime_error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum ApplicationError {
    TransactionProcessorError(TransactionProcessorError),

    PackageError(PackageError),

    SystemError(SystemError),

    ResourceManagerError(ResourceManagerError),

    ComponentError(ComponentError),

    BucketError(BucketError),

    ProofError(ProofError),

    VaultError(VaultError),

    WorktopError(WorktopError),

    AuthZoneError(AuthZoneError),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
