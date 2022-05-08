use sbor::*;
use scrypto::engine::types::*;
use scrypto::rust::fmt;
use scrypto::rust::string::String;
use scrypto::values::*;

use crate::engine::*;
use crate::model::*;

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
    /// Assertion check failed.
    AssertionFailed,

    /// The data is not a valid SBOR value.
    ParseScryptoValueError(ParseScryptoValueError),

    /// Not a valid ABI.
    AbiValidationError(DecodeError),

    AuthZoneDoesNotExist,
    WorktopDoesNotExist,

    /// Failed to allocate an ID.
    IdAllocatorError(IdAllocatorError),

    /// Error when invoking an export.
    InvokeError,

    /// Error when accessing the program memory.
    MemoryAccessError,

    /// Error when allocating memory in program.
    MemoryAllocError,

    /// No return data.
    NoReturnData,

    /// The return value type is invalid.
    InvalidReturnType,

    /// Invalid request code.
    UnknownSystemCall(u32),

    /// Invalid request data.
    InvalidRequestData(DecodeError),

    /// The requested host function does not exist.
    HostFunctionNotFound(usize),

    /// Package does not exist.
    PackageNotFound(PackageAddress),

    PackageError(PackageError),
    SystemError(SystemError),

    /// Blueprint does not exist.
    BlueprintNotFound(PackageAddress, String),

    /// System call not allowed in given context.
    IllegalSystemCall,

    ComponentReentrancy(ComponentAddress),

    /// Component does not exist.
    ComponentNotFound(ComponentAddress),

    /// Component is already loaded
    ComponentAlreadyLoaded(ComponentAddress),

    /// Resource manager does not exist.
    ResourceManagerNotFound(ResourceAddress),

    /// Non-fungible does not exist.
    NonFungibleNotFound(NonFungibleAddress),

    /// Non-fungible already exists.
    NonFungibleAlreadyExists(NonFungibleAddress),

    /// Lazy map does not exist.
    LazyMapNotFound(LazyMapId),

    /// Lazy map removed.
    LazyMapRemoved(LazyMapId),

    /// Duplicate LazyMap added
    DuplicateLazyMap(LazyMapId),

    /// Cyclic LazyMap added
    CyclicLazyMap(LazyMapId),

    /// Vault does not exist.
    VaultNotFound(VaultId),

    /// Vault removed.
    VaultRemoved(VaultId),

    /// Duplicate Vault added
    DuplicateVault(VaultId),

    /// Bucket does not exist.
    BucketNotFound(BucketId),

    /// Proof does not exist.
    ProofNotFound(ProofId),

    /// The bucket contains no resource.
    EmptyProof,

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

    /// Interpreter is not started.
    InterpreterNotStarted,

    /// Invalid log level.
    InvalidLevel,

    /// Resource check failure.
    ResourceCheckFailure,

    /// AuthZone error
    AuthZoneError(AuthZoneError),

    /// System Authorization Failure
    AuthorizationError {
        function: String,
        authorization: MethodAuthorization,
        error: MethodAuthorizationError,
    },

    /// Index out of bounds.
    IndexOutOfBounds {
        index: usize,
        max: usize,
    },

    /// Can't move a locked bucket.
    CantMoveLockedBucket,

    /// Can't move restricted proof.
    CantMoveRestrictedProof(ProofId),

    InvalidInvocation,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// TODO: remove dependency on wasmi
impl wasmi::HostError for RuntimeError {}
