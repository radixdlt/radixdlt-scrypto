use sbor::*;
use scrypto::engine::types::*;
use scrypto::rust::fmt;
use scrypto::rust::string::String;
use scrypto::values::*;
use wasmi::*;

use crate::engine::*;
use crate::model::*;

/// Error coming from WASMI module which maps to wasmi:Error but is cloneable
#[derive(Debug, PartialEq, Clone)]
pub enum WasmiError {
    /// Module validation error. Might occur only at load time.
    Validation(String),
    /// Error while instantiating a module. Might occur when provided
    /// with incorrect exports (i.e. linkage failure).
    Instantiation(String),
    /// Function-level error.
    Function(String),
    /// Table-level error.
    Table(String),
    /// Memory-level error.
    Memory(String),
    /// Global-level error.
    Global(String),
    /// Value-level error.
    Value(String),
    /// Trap.
    Trap,
    /// Custom embedder error.
    Host,
}

impl From<wasmi::Error> for WasmiError {
    fn from(e: Error) -> Self {
        match e {
            Error::Validation(e) => WasmiError::Validation(e),
            Error::Instantiation(e) => WasmiError::Instantiation(e),
            Error::Function(e) => WasmiError::Function(e),
            Error::Table(e) => WasmiError::Table(e),
            Error::Memory(e) => WasmiError::Memory(e),
            Error::Global(e) => WasmiError::Global(e),
            Error::Value(e) => WasmiError::Value(e),
            Error::Trap(_) => WasmiError::Trap,
            Error::Host(_) => WasmiError::Host,
        }
    }
}

/// Represents an error when validating a WASM file.
#[derive(Debug, PartialEq, Clone)]
pub enum WasmValidationError {
    /// The wasm module is invalid.
    InvalidModule,
    /// The wasm module contains a start function.
    StartFunctionNotAllowed,
    /// The wasm module uses float points.
    FloatingPointNotAllowed,
    /// The wasm module does not have memory export.
    NoValidMemoryExport,
    /// package_init function does not exist in module
    NoPackageInitExport(WasmiError),
    /// package_init function is not the correct interface
    InvalidPackageInit,
}

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

    /// The data is not a valid WASM module.
    WasmValidationError(WasmValidationError),

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
    InvalidRequestCode(u32),

    /// Invalid request data.
    InvalidRequestData(DecodeError),

    /// The requested host function does not exist.
    HostFunctionNotFound(usize),

    /// Package does not exist.
    PackageNotFound(PackageAddress),

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
    AuthorizationError(String, MethodAuthorizationError),

    /// Index out of bounds.
    IndexOutOfBounds {
        index: usize,
        max: usize,
    },

    /// Can't move a locked bucket.
    CantMoveLockedBucket,

    /// Can't move restricted proof.
    CantMoveRestrictedProof(ProofId),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl HostError for RuntimeError {}
