use std::fmt;

use sbor::*;
use scrypto::types::*;
use wasmi::*;

use crate::model::*;

#[derive(Debug)]
pub enum RuntimeError {
    BlueprintNotFound,

    InvalidModule(Error),

    UnableToInstantiate(Error),

    HasStartFunction,

    NoValidMemoryExport,

    InvokeError(Error),

    MemoryAccessError(Error),

    NoValidBlueprintReturn,

    InvalidOpCode(u32),

    InvalidRequest(DecodeError),

    UnknownHostFunction(usize),

    UnableToAllocateMemory,

    ResourceLeak(Vec<BID>),

    BlueprintAlreadyExists(Address),

    ComponentAlreadyExists(Address),

    ResourceAlreadyExists(Address),

    ComponentNotFound(Address),

    ResourceNotFound(Address),

    ImmutableResource,

    NotAuthorizedToMint,

    BucketNotFound,

    BucketRefNotFound,

    AccountingError(BucketError),

    UnauthorizedToWithdraw,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl HostError for RuntimeError {}
