use crate::api::types::*;
use crate::scrypto;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;
use crate::data::ScryptoValue;

#[derive(Debug)]
#[scrypto(Categorize, Encode, Decode)]
pub enum RadixEngineInput {
    // High Level method call
    InvokeMethod(Receiver, String, ScryptoValue),
    // Low Level call
    Invoke(CallTableInvocation),

    CreateNode(ScryptoRENode),
    GetVisibleNodeIds(),
    DropNode(RENodeId),

    LockSubstate(RENodeId, SubstateOffset, bool),
    DropLock(LockHandle),
    Read(LockHandle),
    Write(LockHandle, Vec<u8>),

    GetActor(),
}
