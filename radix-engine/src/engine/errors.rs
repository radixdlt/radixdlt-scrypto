use sbor::rust::boxed::Box;
use sbor::rust::fmt;
use sbor::rust::string::String;
use sbor::DecodeError;
use scrypto::engine::types::*;
use scrypto::values::*;

use crate::engine::*;
use crate::model::*;
use crate::wasm::InvokeError;

/// Represents an error when validating a transaction.
#[derive(Debug, PartialEq, Eq)]
pub enum TransactionValidationError {
    ParseScryptoValueError(ParseScryptoValueError),
    IdValidatorError(IdValidatorError),
    VaultNotAllowed(VaultId),
    LazyMapNotAllowed(LazyMapId),
    InvalidSignature,
}

/// Represents an error when executing a transaction.
#[derive(Debug, PartialEq, Clone)]
pub enum RuntimeError {
    /// Error when invoking a blueprint or component (recursive).
    InvokeError(Box<InvokeError>),

    /// The data is not a valid SBOR value.
    ParseScryptoValueError(ParseScryptoValueError),

    AuthZoneDoesNotExist,

    WorktopDoesNotExist,

    /// Failed to allocate an ID.
    IdAllocatorError(IdAllocatorError),

    /// Invalid request code.
    UnknownMethod(String),

    /// Package does not exist.
    PackageNotFound(PackageAddress),
    InvalidPackage(DecodeError),

    PackageError(PackageError),

    SystemError(SystemError),

    /// Blueprint does not exist.
    BlueprintNotFound(PackageAddress, String),

    ComponentReentrancy(ComponentAddress),

    /// Component does not exist.
    ComponentNotFound(ComponentAddress),

    /// Resource manager does not exist.
    ResourceManagerNotFound(ResourceAddress),

    /// Lazy map does not exist.
    LazyMapNotFound(LazyMapId),

    /// Lazy map removed.
    LazyMapRemoved(LazyMapId),

    /// Cyclic LazyMap added
    CyclicLazyMap(LazyMapId),

    /// Vault does not exist.
    VaultNotFound(VaultId),

    /// Vault removed.
    VaultRemoved(VaultId),

    /// Bucket does not exist.
    BucketNotFound(BucketId),

    /// Proof does not exist.
    ProofNotFound(ProofId),

    /// Resource manager access error.
    ResourceManagerError(ResourceManagerError),

    /// Bucket access error.
    BucketError(BucketError),

    /// Vault access error.
    VaultError(VaultError),

    /// Worktop access error.
    WorktopError(WorktopError),

    /// Error when generating or accessing proof.
    ProofError(ProofError),

    /// Bucket is not allowed.
    BucketNotAllowed,

    /// Proof is not allowed.
    ProofNotAllowed,

    /// Vault is not allowed
    VaultNotAllowed,

    /// Lazy Map is not allowed
    LazyMapNotAllowed,

    /// Resource check failure.
    ResourceCheckFailure(ResourceFailure),

    /// AuthZone error
    AuthZoneError(AuthZoneError),

    /// System Authorization Failure
    AuthorizationError {
        function: String,
        authorization: MethodAuthorization,
        error: MethodAuthorizationError,
    },

    /// Can't move a locked bucket.
    CantMoveLockedBucket,

    /// Can't move restricted proof.
    CantMoveRestrictedProof(ProofId),

    InvalidInvocation,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ResourceFailure {
    Resource(ResourceAddress),
    Resources(Vec<ResourceAddress>),
    UnclaimedLazyMap,
    Unknown,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
