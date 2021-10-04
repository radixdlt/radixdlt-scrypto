use sbor::*;
use scrypto::rust::fmt;
use scrypto::types::*;
use wasmi::*;

use crate::model::*;

/// Represents an error occurred during transaction execution.
#[derive(Debug)]
pub enum RuntimeError {
    InvalidModule(Error),

    StartFunctionNotAllowed,

    FloatingPointNotAllowed,

    NoValidMemoryExport,

    InvokeError(Error),

    MemoryAccessError(Error),

    NoReturnData,

    InvalidReturnType,

    InvalidOpCode(u32),

    InvalidRequest(DecodeError),

    InvalidData(DecodeError),

    UnknownHostFunction(usize),

    UnableToAllocateMemory,

    ResourceCheckFailure,

    PackageAlreadyExists(Address),

    ComponentAlreadyExists(Address),

    ResourceDefAlreadyExists(Address),

    PackageNotFound(Address),

    ComponentNotFound(Address),

    LazyMapNotFound(Mid),

    ResourceDefNotFound(Address),

    UnableToMintDueToFixedSupply,

    UnauthorizedToMint,

    VaultNotFound(Vid),

    BucketNotFound(Bid),

    ReferenceNotFound(Rid),

    AccountingError(BucketError),

    UnauthorizedAccess,

    BucketNotAllowed,

    ReferenceNotAllowed,

    VmNotStarted,

    InvalidLogLevel,

    BucketNotReserved,

    ReferenceNotReserved,

    UnexpectedBucketReturn,

    UnexpectedReferenceReturn,

    InvalidAddressType,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl HostError for RuntimeError {}
