use crate::values::ScryptoValue;
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::scrypto_encode;
use scrypto::core::*;
use scrypto::engine::api::ScryptoSyscalls;
use scrypto::engine::types::*;

#[cfg(target_arch = "wasm32")]
extern "C" {
    pub fn radix_engine(input: *mut u8) -> *mut u8;
}

/// Utility function for making a radix engine call.
#[cfg(target_arch = "wasm32")]
pub fn call_engine<V: Decode>(input: RadixEngineInput) -> V {
    use crate::buffer::{scrypto_decode_from_buffer, *};

    unsafe {
        let input_ptr = scrypto_encode_to_buffer(&input);
        let output_ptr = radix_engine(input_ptr);
        scrypto_decode_from_buffer::<V>(output_ptr).unwrap()
    }
}

/// Utility function for making a radix engine call.
#[cfg(not(target_arch = "wasm32"))]
pub fn call_engine<V: Decode>(_input: RadixEngineInput) -> V {
    todo!()
}


#[derive(Debug, TypeId, Encode, Decode)]
pub struct SyscallError;

pub struct Syscalls;

impl ScryptoSyscalls<SyscallError> for Syscalls {
    fn sys_invoke_scrypto_function<ARGS: Encode, V: Decode>(
        &mut self,
        fn_ident: ScryptoFunctionIdent,
        args: &ARGS,
    ) -> Result<V, SyscallError> {
        let rtn = call_engine(
            RadixEngineInput::InvokeScryptoFunction(fn_ident, scrypto_encode(args)));
        Ok(rtn)
    }

    fn sys_invoke_scrypto_method<ARGS: Encode, V: Decode>(
        &mut self,
        method_ident: ScryptoMethodIdent,
        args: &ARGS,
    ) -> Result<V, SyscallError> {
        let rtn = call_engine(RadixEngineInput::InvokeScryptoMethod(method_ident, scrypto_encode(args)));
        Ok(rtn)
    }

    fn sys_invoke_native_function<ARGS: Encode, V: Decode>(
        &mut self,
        native_function: NativeFunction,
        args: &ARGS,
    ) -> Result<V, SyscallError> {
        let rtn = call_engine(RadixEngineInput::InvokeNativeFunction(
            native_function,
            scrypto_encode(args),
        ));
        Ok(rtn)
    }

    fn sys_invoke_native_method<ARGS: Encode, V: Decode>(
        &mut self,
        native_method: NativeMethod,
        args: &ARGS,
    ) -> Result<V, SyscallError> {
        let rtn = call_engine(RadixEngineInput::InvokeNativeMethod(
            native_method,
            scrypto_encode(args),
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

    fn sys_lock_substate(
        &mut self,
        node_id: RENodeId,
        offset: SubstateOffset,
        mutable: bool,
    ) -> Result<LockHandle, SyscallError> {
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


#[macro_export]
macro_rules! native_methods {
    ($type_ident:expr => { $($vis:vis $fn:ident $method_name:ident ($($args:tt)*) -> $rtn:ty { $fn_ident:expr, $arg:expr })* } ) => {
        $(
            $vis $fn $method_name ($($args)*) -> $rtn {
                let input = RadixEngineInput::InvokeNativeMethod(
                    $type_ident($fn_ident),
                    scrypto::buffer::scrypto_encode(&$arg)
                );
                call_engine(input)
            }
        )+
    };
}
