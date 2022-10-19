use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::{Decode, Encode, TypeId};

use super::types::*;

#[cfg(target_arch = "wasm32")]
extern "C" {
    pub fn radix_engine(input: *mut u8) -> *mut u8;
}

#[derive(Debug, TypeId, Encode, Decode)]
pub enum RadixEngineInput {
    InvokeScryptoFunction(ScryptoFunctionIdent, Vec<u8>),
    InvokeScryptoMethod(ScryptoMethodIdent, Vec<u8>),
    InvokeNativeFunction(NativeFunctionIdent, Vec<u8>),
    InvokeNativeMethod(NativeMethodIdent, Vec<u8>),

    RENodeCreate(ScryptoRENode),
    RENodeGlobalize(RENodeId),
    GetVisibleNodeIds(),

    LockSubstate(RENodeId, SubstateOffset, bool),
    DropLock(LockHandle),
    Read(LockHandle),
    Write(LockHandle, Vec<u8>),

    GetActor(),
    EmitLog(Level, String),
    GenerateUuid(),
}
