use crate::api::types::{
    FnIdentifier, LockHandle, RENodeId, ScryptoRENode, ScryptoReceiver, SubstateOffset,
};
use crate::api::Invocation;
use crate::data::{ScryptoDecode, ScryptoEncode};
use crate::model::CallTableInvocation;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

pub trait SerializableInvocation:
    Into<CallTableInvocation> + Invocation<Output = Self::ScryptoOutput>
{
    type ScryptoOutput: ScryptoDecode;
}

pub trait EngineWasmApi {
    const ID: u8 = 0;
    type Input: ScryptoEncode;
    type Output: ScryptoDecode;
}

pub struct InvokeMethod;

impl EngineWasmApi for InvokeMethod {
    const ID: u8 = 1;
    type Input = (ScryptoReceiver, String, Vec<u8>);
    type Output = Vec<u8>;
}

pub struct Invoke;

impl EngineWasmApi for Invoke {
    const ID: u8 = 2;
    type Input = CallTableInvocation;
    type Output = Vec<u8>;
}

pub struct CreateNode;

impl EngineWasmApi for CreateNode {
    const ID: u8 = 3;
    type Input = ScryptoRENode;
    type Output = RENodeId;
}

pub struct GetVisibleNodeIds;

impl EngineWasmApi for GetVisibleNodeIds {
    const ID: u8 = 4;
    type Input = ();
    type Output = Vec<RENodeId>;
}

pub struct DropNode;

impl EngineWasmApi for DropNode {
    const ID: u8 = 5;
    type Input = RENodeId;
    type Output = ();
}

pub struct LockSubstate;

impl EngineWasmApi for LockSubstate {
    const ID: u8 = 6;
    type Input = (RENodeId, SubstateOffset, bool);
    type Output = LockHandle;
}

pub struct DropLock;

impl EngineWasmApi for DropLock {
    const ID: u8 = 7;
    type Input = LockHandle;
    type Output = ();
}

pub struct Read;

impl EngineWasmApi for Read {
    const ID: u8 = 8;
    type Input = LockHandle;
    type Output = Vec<u8>;
}

pub struct Write;

impl EngineWasmApi for Write {
    const ID: u8 = 9;
    type Input = (LockHandle, Vec<u8>);
    type Output = ();
}

pub struct GetActor;

impl EngineWasmApi for GetActor {
    const ID: u8 = 10;
    type Input = ();
    type Output = FnIdentifier;
}
