use transaction::errors::*;

use crate::engine::REActor;
use crate::fee::FeeReserveError;
use crate::model::*;
use crate::types::*;
use crate::wasm::InvokeError;

use super::ModuleError;

/// Represents an error when executing a transaction.
#[derive(Debug)]
pub enum RuntimeError {
    // TODO: better abstraction
    KernelModuleError(ModuleError),

    /// Error when invoking a blueprint or component (recursive).
    InvokeError(Box<InvokeError>),

    /// The data is not a valid SBOR value.
    DecodeError(DecodeError),

    AuthZoneDoesNotExist,

    WorktopDoesNotExist,

    /// Failed to allocate an ID.
    IdAllocationError(IdAllocationError),

    /// Invalid request code.
    MethodDoesNotExist(FnIdentifier),
    InvalidFnInput {
        fn_identifier: FnIdentifier,
    },
    InvalidFnOutput {
        fn_identifier: FnIdentifier,
        output: Value,
    },

    /// Package does not exist.
    PackageNotFound(PackageAddress),
    InvalidPackage(DecodeError),

    PackageError(PackageError),

    SystemError(SystemError),

    /// Blueprint does not exist.
    BlueprintNotFound(PackageAddress, String),

    Reentrancy(SubstateId),
    PackageReentrancy,

    ComponentDecodeError(DecodeError),

    /// Resource manager does not exist.
    ResourceManagerNotFound(ResourceAddress),

    SubstateReadNotReadable(REActor, SubstateId),
    SubstateWriteNotWriteable(REActor, SubstateId),
    SubstateReadSubstateNotFound(SubstateId),
    RENodeNotFound(RENodeId),

    MovingInvalidType,
    StoredNodeRemoved(RENodeId),
    StoredNodeChangedChildren,

    /// Bucket does not exist.
    BucketNotFound(BucketId),

    /// Proof does not exist.
    ProofNotFound(ProofId),

    /// Resource manager access error.
    ResourceManagerError(ResourceManagerError),
    ComponentError(ComponentError),

    /// Bucket access error.
    BucketError(BucketError),

    /// Vault access error.
    VaultError(VaultError),

    /// Worktop access error.
    WorktopError(WorktopError),

    /// Error when generating or accessing proof.
    ProofError(ProofError),

    ValueNotAllowed,

    /// Bucket is not allowed.
    BucketNotAllowed,

    /// Proof is not allowed.
    ProofNotAllowed,

    /// Vault is not allowed
    VaultNotAllowed,

    /// Key Value store is not allowed
    KeyValueStoreNotAllowed,

    /// Resource check failure.
    DropFailure(DropFailure),

    /// AuthZone error
    AuthZoneError(AuthZoneError),

    /// System Authorization Failure
    AuthorizationError {
        function: FnIdentifier,
        authorization: MethodAuthorization,
        error: MethodAuthorizationError,
    },

    /// Can't move a locked bucket.
    CantMoveLockedBucket,

    /// Can't move restricted proof.
    CantMoveRestrictedProof,
    CantMoveWorktop,
    CantMoveAuthZone,

    RENodeGlobalizeTypeNotAllowed(RENodeId),
    RENodeCreateInvalidPermission,
    RENodeCreateNodeNotFound(RENodeId),

    InvokeMethodInvalidReceiver(RENodeId),
    InvokeMethodInvalidReferencePass(RENodeId),
    InvokeMethodInvalidReferenceReturn(RENodeId),

    NotSupported,

    CostingError(FeeReserveError),

    MaxCallDepthLimitReached,

    LockFeeError(LockFeeError),
}

#[derive(Debug, PartialEq)]
pub enum LockFeeError {
    RENodeNotInTrack,
    RENodeAlreadyTouched,
    RENodeNotFound,
    NotRadixToken,
    InsufficientBalance,
}

#[derive(Debug, PartialEq)]
pub enum DropFailure {
    System,
    Resource,
    Component,
    Bucket,
    Worktop,
    Vault,
    Package,
    KeyValueStore,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
