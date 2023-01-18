use crate::api::types::*;
use crate::data::ScryptoValue;
use crate::*;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum RadixEngineInput {
    // High Level method call
    InvokeMethod(ScryptoReceiver, String, ScryptoValue),
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
