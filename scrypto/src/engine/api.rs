use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::{Decode, Encode, TypeId};
use scrypto::core::*;
use scrypto::engine::types::*;
use crate::engine::utils::ScryptoSyscalls;

#[cfg(target_arch = "wasm32")]
use crate::engine::utils::call_engine;

use super::types::*;

#[cfg(target_arch = "wasm32")]
extern "C" {
    pub fn radix_engine(input: *mut u8) -> *mut u8;
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, TypeId, Encode, Decode)]
pub struct SyscallError;

#[cfg(target_arch = "wasm32")]
pub struct Syscalls;

#[cfg(target_arch = "wasm32")]
impl ScryptoSyscalls<SyscallError> for Syscalls {
    fn sys_invoke_scrypto_function<V: Decode>(&mut self, fn_ident: ScryptoFunctionIdent, args: Vec<u8>) -> Result<V, SyscallError> {
        let rtn = call_engine(RadixEngineInput::InvokeScryptoFunction(
            fn_ident, args
        ));
        Ok(rtn)
    }

    fn sys_invoke_scrypto_method<V: Decode>(&mut self, method_ident: ScryptoMethodIdent, args: Vec<u8>) -> Result<V, SyscallError> {
        let rtn = call_engine(RadixEngineInput::InvokeScryptoMethod(
            method_ident, args
        ));
        Ok(rtn)
    }

    fn sys_invoke_native_function<V: Decode>(&mut self, native_function: NativeFunction, args: Vec<u8>) -> Result<V, SyscallError> {
        let rtn = call_engine(RadixEngineInput::InvokeNativeFunction(
            native_function, args
        ));
        Ok(rtn)
    }

    fn sys_invoke_native_method<V: Decode>(&mut self, native_method: NativeMethod, receiver: RENodeId, args: Vec<u8>) -> Result<V, SyscallError> {
        let rtn = call_engine(RadixEngineInput::InvokeNativeMethod(
            native_method, receiver, args
        ));
        Ok(rtn)
    }

    fn sys_create_node(&mut self, node: ScryptoRENode) -> Result<RENodeId, SyscallError> {
        let rtn = call_engine(RadixEngineInput::CreateNode(node));
        Ok(rtn)
    }

    fn sys_drop_node(&mut self, node_id: RENodeId) -> Result<(), SyscallError> {
        let rtn = call_engine(RadixEngineInput::DropNode(node_id));
        Ok(rtn)
    }

    fn sys_get_visible_nodes(&mut self) -> Result<Vec<RENodeId>, SyscallError> {
        let rtn = call_engine(RadixEngineInput::GetVisibleNodeIds());
        Ok(rtn)
    }

    fn sys_lock_substate(&mut self, node_id: RENodeId, offset: SubstateOffset, mutable: bool) -> Result<LockHandle, SyscallError> {
        let rtn = call_engine(RadixEngineInput::LockSubstate(node_id, offset, mutable));
        Ok(rtn)
    }

    fn sys_read<V: Decode>(&mut self, lock_handle: LockHandle) -> Result<V, SyscallError> {
        let rtn = call_engine(RadixEngineInput::Read(lock_handle));
        Ok(rtn)
    }

    fn sys_write(&mut self, lock_handle: LockHandle, buffer: Vec<u8>) -> Result<(), SyscallError> {
        let rtn = call_engine(RadixEngineInput::Write(lock_handle, buffer));
        Ok(rtn)
    }

    fn sys_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), SyscallError> {
        let rtn = call_engine(RadixEngineInput::DropLock(lock_handle));
        Ok(rtn)
    }

    fn sys_get_actor(&mut self) -> Result<ScryptoActor, SyscallError> {
        let rtn = call_engine(RadixEngineInput::GetActor());
        Ok(rtn)
    }

    fn sys_generate_uuid(&mut self) -> Result<u128, SyscallError> {
        let rtn = call_engine(RadixEngineInput::GenerateUuid());
        Ok(rtn)
    }

    fn sys_emit_log(&mut self, level: Level, message: String) -> Result<(), SyscallError> {
        let rtn = call_engine(RadixEngineInput::EmitLog(level, message));
        Ok(rtn)
    }
}


#[derive(Debug, TypeId, Encode, Decode)]
pub enum RadixEngineInput {
    InvokeScryptoFunction(ScryptoFunctionIdent, Vec<u8>),
    InvokeScryptoMethod(ScryptoMethodIdent, Vec<u8>),
    InvokeNativeFunction(NativeFunction, Vec<u8>),
    InvokeNativeMethod(NativeMethod, RENodeId, Vec<u8>),

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
