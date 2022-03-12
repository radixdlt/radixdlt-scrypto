use sbor::*;
use scrypto::engine::types::*;
use scrypto::prelude::NonFungibleAddress;
use scrypto::rust::fmt;
use scrypto::types::*;
use wasmi::*;

use crate::engine::*;
use crate::model::*;

/// Represents an error when validating a WASM file.
#[derive(Debug, PartialEq)]
pub enum WasmValidationError {
    /// The wasm module is invalid.
    InvalidModule(),

    /// The wasm module contains a start function.
    StartFunctionNotAllowed,

    /// The wasm module uses float points.
    FloatingPointNotAllowed,

    /// The wasm module does not have memory export.
    NoValidMemoryExport,
}

/// Represents an error when parsing a value from a byte array.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataValidationError {
    DecodeError(DecodeError),
    CustomValueValidatorError(CustomValueValidatorError),
}

/// Represents an error when validating a transaction.
#[derive(Debug, PartialEq, Eq)]
pub enum TransactionValidationError {
    DataValidationError(DataValidationError),
    IdValidatorError(IdValidatorError),
    VaultNotAllowed(VaultId),
    LazyMapNotAllowed(LazyMapId),
    InvalidSignature,
    UnexpectedEnd,
}

/// Represents an error when executing a transaction.
#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    /// Assertion check failed.
    AssertionFailed,

    /// The data is not a valid WASM module.
    WasmValidationError(WasmValidationError),

    /// The data is not a valid SBOR value.
    DataValidationError(DataValidationError),

    /// Not a valid ABI.
    AbiValidationError(DecodeError),

    /// Failed to allocate an ID.
    IdAllocatorError(IdAllocatorError),

    /// Error when invoking an export.
    InvokeError(),

    /// Error when accessing the program memory.
    MemoryAccessError(),

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
    PackageNotFound(PackageId),

    /// System call not allowed in given context.
    IllegalSystemCall(),

    /// No component has been loaded.
    ComponentNotLoaded(),

    /// Component does not exist.
    ComponentNotFound(ComponentId),

    /// Component is already loaded
    ComponentAlreadyLoaded(ComponentId),

    /// Resource definition does not exist.
    ResourceDefNotFound(ResourceDefId),

    /// Non-fungible does not exist.
    NonFungibleNotFound(NonFungibleAddress),

    /// Non-fungible already exists.
    NonFungibleAlreadyExists(NonFungibleAddress),

    /// Lazy map does not exist.
    LazyMapNotFound(LazyMapId),

    /// Lazy map removed.
    LazyMapRemoved(),

    /// Duplicate LazyMap added
    DuplicateLazyMap(LazyMapId),

    /// Cyclic LazyMap added
    CyclicLazyMap(),

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

    /// Bucket access error.
    BucketError(BucketError),

    /// Resource definition access error.
    ResourceDefError(ResourceDefError),

    /// Vault access error.
    VaultError(VaultError),

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

    /// Index out of bounds.
    IndexOutOfBounds { index: usize, max: usize },
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl HostError for RuntimeError {}
