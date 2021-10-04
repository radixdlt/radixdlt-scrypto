use sbor::*;
use scrypto::rust::fmt;
use scrypto::types::*;
use wasmi::*;

use crate::model::*;

/// Represents an error occurred during transaction execution.
#[derive(Debug)]
pub enum RuntimeError {
    /// The wasm module is invalid.
    InvalidModule(Error),

    /// The wasm module contains a start function.
    StartFunctionNotAllowed,

    /// The wasm module uses float points.
    FloatingPointNotAllowed,

    /// The wasm module does not have memory export.
    NoValidMemoryExport,

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

    /// The data is invalid (not in SBOR format).
    InvalidData(DecodeError),

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

    /// Lazy map does not exist.
    LazyMapNotFound(Mid),

    /// Vault does not exist.
    VaultNotFound(Vid),

    /// Bucket does not exist.
    BucketNotFound(Bid),

    /// Bucket ref does not exist.
    BucketRefNotFound(Rid),

    /// Tried to mint resource with fixed supply.
    MintFixedSupplyResource,

    /// Unauthorized mint.
    UnauthorizedMint,

    /// Unauthorized access.
    UnauthorizedAccess,

    /// Accounting error.
    AccountingError(BucketError),

    /// Bucket is not allowed (in component state).
    BucketNotAllowed,

    /// Bucket ref is not allowed (in component state).
    BucketRefNotAllowed,

    /// Interpreter is not started.
    InterpreterNotStarted,

    /// Invalid log level.
    InvalidLogLevel,

    /// Invalid access type.
    InvalidAddressType,

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
