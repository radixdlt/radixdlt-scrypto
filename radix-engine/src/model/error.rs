use sbor::*;
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
#[derive(Debug)]
pub enum DataValidationError {
    DecodeError(DecodeError),
    InvalidTypeId(u8),
    InvalidDecimal(ParseDecimalError),
    InvalidBigDecimal(ParseBigDecimalError),
    InvalidAddress(ParseAddressError),
    InvalidH256(ParseH256Error),
    InvalidBid(ParseBidError),
    InvalidRid(ParseRidError),
    InvalidMid(ParseMidError),
    InvalidVid(ParseVidError),
}

/// Represents an error when validating a transaction.
#[derive(Debug)]
pub enum TransactionValidationError {
    DataValidationError(DataValidationError),
    IdAllocatorError(IdAllocatorError),
    TempBucketNotFound(Bid),
    TempBucketRefNotFound(Rid),
    TempBucketLocked(Bid),
    InvalidSignature,
    UnexpectedEnd,
}

/// Represents an error when executing a transaction.
#[derive(Debug)]
pub enum RuntimeError {
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
    PackageAlreadyExists(Address),

    /// Component already exists.
    ComponentAlreadyExists(Address),

    /// Resource definition already exists.
    ResourceDefAlreadyExists(Address),

    /// Resource definition already exists.
    LazyMapAlreadyExists(Mid),

    /// Package does not exist.
    PackageNotFound(Address),

    /// Component does not exist.
    ComponentNotFound(Address),

    /// Resource definition does not exist.
    ResourceDefNotFound(Address),

    /// Nft does not exist.
    NftNotFound(Address, u128),

    /// Nft already exists.
    NftAlreadyExists(Address, u128),

    /// Lazy map does not exist.
    LazyMapNotFound(Mid),

    /// Vault does not exist.
    VaultNotFound(Vid),

    /// Bucket does not exist.
    BucketNotFound(Bid),

    /// Bucket ref does not exist.
    BucketRefNotFound(Rid),

    /// Not a package address.
    InvalidPackageAddress(Address),

    /// Not a component address.
    InvalidComponentAddress(Address),

    /// Not a resource def address.
    InvalidResourceDefAddress(Address),

    /// The referenced bucket contains no resource.
    EmptyBucketRef,

    /// Bucket access error.
    BucketError(BucketError),

    /// Component access error.
    ComponentError(ComponentError),

    /// Lazy map access error.
    LazyMapError(LazyMapError),

    /// Bucket ref access error.
    ResourceDefError(ResourceDefError),

    /// Vault access error.
    VaultError(VaultError),

    /// Nft access error.
    NftError(NftError),

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
    InvalidLogLevel,

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
