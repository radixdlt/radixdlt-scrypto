use crate::engine::node_move_module::NodeMoveError;
use crate::engine::{AuthError, ExecutionMode, LockFlags, ResolvedActor};
use radix_engine_interface::api::types::{GlobalAddress, LockHandle, RENodeId, SubstateOffset};
use radix_engine_interface::data::ReadOwnedNodesError;
use sbor::*;

use crate::model::*;
use crate::types::*;
use crate::wasm::WasmRuntimeError;

use super::TrackError;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum IdAllocationError {
    RENodeIdWasNotAllocated(RENodeId),
    AllocatedIDsNotEmpty,
    OutOfID,
}

/// Represents an error which causes a tranasction to be rejected.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum RejectionError {
    SuccessButFeeLoanNotRepaid,
    ErrorBeforeFeeLoanRepaid(RuntimeError),
    TransactionEpochNotYetValid {
        valid_from: u64,
        current_epoch: u64,
    },
    TransactionEpochNoLongerValid {
        valid_until: u64,
        current_epoch: u64,
    },
}

impl fmt::Display for RejectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Represents an error when executing a transaction.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum KernelError {
    InvalidModeTransition(ExecutionMode, ExecutionMode),

    // invocation
    WasmRuntimeError(WasmRuntimeError),

    InvalidReferenceWrite(GlobalAddress),

    RENodeNotFound(RENodeId),

    InvalidScryptoFnOutput,

    // ID allocation
    IdAllocationError(IdAllocationError),

    // SBOR decoding
    SborDecodeError(DecodeError),
    SborEncodeError(EncodeError),
    ReadOwnedNodesError(ReadOwnedNodesError), // semantic error

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
    InvalidId(RENodeId),

    // Actor Constraints
    InvalidDropNodeVisibility {
        mode: ExecutionMode,
        actor: ResolvedActor,
        node_id: RENodeId,
    },
    InvalidCreateNodeVisibility {
        mode: ExecutionMode,
        actor: ResolvedActor,
    },
    InvalidSubstateVisibility {
        mode: ExecutionMode,
        actor: ResolvedActor,
        node_id: RENodeId,
        offset: SubstateOffset,
        flags: LockFlags,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Categorize)]
pub enum ScryptoFnResolvingError {
    BlueprintNotFound,
    MethodNotFound,
    InvalidInput,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum InterpreterError {
    InvalidInvocation,

    InvalidScryptoInvocation(PackageAddress, String, String, ScryptoFnResolvingError),
    InvalidScryptoReturn(DecodeError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ModuleError {
    NodeMoveError(NodeMoveError),
    AuthError(AuthError),
    CostingError(CostingError),
    RoyaltyError(RoyaltyError),
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

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ApplicationError {
    TransactionProcessorError(TransactionProcessorError),

    PackageError(PackageError),

    EpochManagerError(EpochManagerError),

    ValidatorError(ValidatorError),

    ResourceManagerError(ResourceManagerError),

    AccessRulesChainError(AccessRulesChainError),

    TransactionRuntimeError(TransactionRuntimeError),

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
