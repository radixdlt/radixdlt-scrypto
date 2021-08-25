use sbor::*;
use scrypto::types::rust::fmt;
use scrypto::types::rust::vec::Vec;
use scrypto::types::*;
use wasmi::*;

use crate::model::*;

/// Represents an error occurred during transaction execution.
#[derive(Debug)]
pub enum RuntimeError {
    InvalidModule(Error),

    UnableToInstantiate(Error),

    StartFunctionNotAllowed,

    FloatingPointNotAllowed,

    NoValidMemoryExport,

    InvokeError(Error),

    MemoryAccessError(Error),

    NoReturnValue,

    InvalidReturnType,

    InvalidOpCode(u32),

    InvalidRequest(DecodeError),

    InvalidData(DecodeError),

    UnknownHostFunction(usize),

    UnableToAllocateMemory,

    ResourceLeak(Vec<BID>, Vec<RID>),

    PackageAlreadyExists(Address),

    ComponentAlreadyExists(Address),

    ResourceAlreadyExists(Address),

    InvalidResourceParameter,

    PackageNotFound(Address),

    ComponentNotFound(Address),

    ResourceNotFound(Address),

    FixedResourceMintNotAllowed,

    UnauthorizedToMint,

    BucketNotFound,

    ReferenceNotFound,

    AccountingError(BucketError),

    UnauthorizedToWithdraw,

    PersistedBucketMoveNotAllowed,

    ReferenceNotAllowed,

    VmNotStarted,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl HostError for RuntimeError {}
