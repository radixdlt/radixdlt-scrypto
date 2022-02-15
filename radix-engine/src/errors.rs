use sbor::*;
use scrypto::engine::types::*;
use scrypto::rust::fmt;
use scrypto::types::*;
use wasmi::*;

use crate::engine::*;
use crate::model::*;

/// Represents an error when validating a WASM file.
#[derive(Debug)]
pub enum WasmValidationError {
    /// The wasm module is invalid.
    InvalidModule(Error),

    /// The wasm module contains a start function.
    StartFunctionNotAllowed,

    /// The wasm module uses float points.
    FloatingPointNotAllowed,

    /// The wasm module does not have memory export.
    NoValidMemoryExport,
}

/// Represents an error when parsing a value from a byte array.
#[derive(Debug, Clone)]
pub enum DataValidationError {
    DecodeError(DecodeError),
    CustomValueValidatorError(CustomValueValidatorError),
}

/// Represents an error when validating a transaction.
#[derive(Debug)]
pub enum TransactionValidationError {
    DataValidationError(DataValidationError),
    IdValidatorError(IdValidatorError),
    InvalidSignature,
    UnexpectedEnd,
}

/// Represents an error when executing a transaction.
#[derive(Debug)]
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
    InvokeError(Error),

    /// Error when accessing the program memory.
    MemoryAccessError(Error),

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

    /// Package already exists.
    PackageAlreadyExists(PackageRef),

    /// Component already exists.
    ComponentAlreadyExists(ComponentRef),

    /// Resource definition already exists.
    ResourceDefAlreadyExists(ResourceDefRef),

    /// Resource definition already exists.
    LazyMapAlreadyExists(LazyMapId),

    /// Package does not exist.
    PackageNotFound(PackageRef),

    /// System call not allowed in given context.
    IllegalSystemCall(),

    /// No component has been loaded.
    ComponentNotLoaded(),

    /// Component does not exist.
    ComponentNotFound(ComponentRef),

    /// Component is already loaded
    ComponentAlreadyLoaded(ComponentRef),

    /// Resource definition does not exist.
    ResourceDefNotFound(ResourceDefRef),

    /// Non-fungible does not exist.
    NonFungibleNotFound(ResourceDefRef, NonFungibleKey),

    /// Non-fungible already exists.
    NonFungibleAlreadyExists(ResourceDefRef, NonFungibleKey),

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

    /// Bucket ref does not exist.
    BucketRefNotFound(BucketRefId),
    /// The referenced bucket contains no resource.
    EmptyBucketRef,

    /// Bucket access error.
    BucketError(BucketError),

    /// Bucket ref access error.
    ResourceDefError(ResourceDefError),

    /// Vault access error.
    VaultError(VaultError),

    /// Bucket is not allowed.
    BucketNotAllowed,

    /// BucketRef is not allowed.
    BucketRefNotAllowed,

    /// Vault is not allowed
    VaultNotAllowed,

    /// Lazy Map is not allowed
    LazyMapNotAllowed,

    /// Interpreter is not started.
    InterpreterNotStarted,

    /// Invalid log level.
    InvalidLevel,

    /// The bucket id is not reserved.
    BucketNotReserved,

    /// The bucket ref id is not reserved.
    BucketRefNotReserved,

    /// Resource check failure.
    ResourceCheckFailure,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl HostError for RuntimeError {}
