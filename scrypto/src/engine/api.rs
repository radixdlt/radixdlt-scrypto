use sbor::rust::string::String;
use sbor::rust::vec::Vec;

use super::types::*;
use crate::scrypto;

#[cfg(target_arch = "wasm32")]
extern "C" {
    pub fn radix_engine(input: *mut u8) -> *mut u8;
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum RadixEngineInput {
    InvokeScryptoFunction(ScryptoFunctionIdent, Vec<u8>),
    InvokeScryptoMethod(ScryptoMethodIdent, Vec<u8>),
    InvokeNativeFunction(NativeFunction, Vec<u8>),
    InvokeNativeMethod(NativeMethod, Vec<u8>),

    CreateNode(ScryptoRENode),
    GetVisibleNodeIds(),
    DropNode(RENodeId),

    LockSubstate(RENodeId, SubstateOffset, bool),
    DropLock(LockHandle),
    Read(LockHandle),
    Write(LockHandle, Vec<u8>),

    GetActor(),
    EmitLog(Level, String),
    GenerateUuid(),
    GetTransactionHash(),
}
