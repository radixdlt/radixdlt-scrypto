use transaction::errors::*;

use crate::engine::REActor;
use crate::fee::FeeReserveError;
use crate::model::*;
use crate::types::*;
use crate::wasm::InvokeError;

#[derive(Debug)]
pub enum RuntimeError {
    /// An error occurred within the kernel.
    KernelError(KernelError),

    /// An error occurred within a kernel module.
    ModuleError(ModuleError),

    /// An error occurred within application logic, like the RE models.
    ApplicationError(ApplicationError),
}

#[derive(Debug)]
pub enum KernelError {
    // invocation
    InvokeError(Box<InvokeError>),
    InvokeMethodInvalidReceiver(RENodeId),
    InvokeMethodInvalidReferencePass(RENodeId),
    InvokeMethodInvalidReferenceReturn(RENodeId),
    MaxCallDepthLimitReached,
    MethodDoesNotExist(FnIdentifier),
    InvalidFnInput {
        fn_identifier: FnIdentifier,
    },
    InvalidFnOutput {
        fn_identifier: FnIdentifier,
        output: Value,
    },

    // ID allocation
    IdAllocationError(IdAllocationError),

    // SBOR decoding
    DecodeError(DecodeError),

    // RENode
    BucketNotFound(BucketId),
    ProofNotFound(ProofId),
    PackageNotFound(PackageAddress),
    BlueprintNotFound(PackageAddress, String),
    ResourceManagerNotFound(ResourceAddress),
    WorktopNotFound,
    RENodeNotFound(RENodeId),
    StoredNodeRemoved(RENodeId),
    RENodeGlobalizeTypeNotAllowed(RENodeId),
    RENodeCreateInvalidPermission,
    RENodeCreateNodeNotFound(RENodeId),
    RENodeAlreadyTouched,
    RENodeNotInTrack,

    // Substate
    Reentrancy(SubstateId),
    SubstateReadNotReadable(REActor, SubstateId),
    SubstateWriteNotWriteable(REActor, SubstateId),
    SubstateReadSubstateNotFound(SubstateId),

    // constraints
    ValueNotAllowed,
    BucketNotAllowed,
    ProofNotAllowed,
    VaultNotAllowed,
    KeyValueStoreNotAllowed,
    CantMoveLockedBucket,
    CantMoveRestrictedProof,
    CantMoveWorktop,
    CantMoveAuthZone,
    DropFailure(DropFailure),
}

#[derive(Debug)]
pub enum ModuleError {
    AuthorizationError {
        function: FnIdentifier,
        authorization: MethodAuthorization,
        error: MethodAuthorizationError,
    },

    CostingError(FeeReserveError),
}

#[derive(Debug)]
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
