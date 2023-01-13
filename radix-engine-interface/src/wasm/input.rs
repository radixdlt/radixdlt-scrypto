use crate::api::types::*;
use crate::*;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum RadixEngineInput {
    Invoke(SerializedInvocation),

    CreateNode(ScryptoRENode),
    GetVisibleNodeIds(),
    DropNode(RENodeId),

    LockSubstate(RENodeId, SubstateOffset, bool),
    DropLock(LockHandle),
    Read(LockHandle),
    Write(LockHandle, Vec<u8>),

    GetActor(),
}
