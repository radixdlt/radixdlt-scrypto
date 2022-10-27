use std::fmt::Debug;
use sbor::Decode;
use scrypto::core::*;
use scrypto::engine::types::*;
use crate::values::ScryptoValue;

use super::api::RadixEngineInput;

/// Utility function for making a radix engine call.
#[cfg(target_arch = "wasm32")]
pub fn call_engine<V: Decode>(input: RadixEngineInput) -> V {
    use crate::buffer::{scrypto_decode_from_buffer, *};
    use crate::engine::api::radix_engine;

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

pub trait ScryptoSyscalls<E: Debug> {
    fn sys_invoke_scrypto_function<V: Decode>(&mut self, fn_ident: ScryptoFunctionIdent, args: Vec<u8>) -> Result<V, E>;
    fn sys_invoke_scrypto_method<V: Decode>(&mut self, method_ident: ScryptoMethodIdent, args: Vec<u8>) -> Result<V, E>;
    fn sys_invoke_native_function<V: Decode>(&mut self, native_function: NativeFunction, args: Vec<u8>) -> Result<V, E>;
    fn sys_invoke_native_method<V: Decode>(&mut self, native_method: NativeMethod, receiver: Receiver, args: Vec<u8>) -> Result<V, E>;
    fn sys_create_node(&mut self, node: ScryptoRENode) -> Result<RENodeId, E>;
    fn sys_get_visible_nodes(&mut self) -> Result<Vec<RENodeId>, E>;
    fn sys_lock_substate(&mut self, node_id: RENodeId, offset: SubstateOffset, mutable: bool) -> Result<LockHandle, E>;
    fn sys_read<V: Decode>(&mut self, lock_handle: LockHandle) -> Result<V, E>;
    fn sys_write(&mut self, lock_handle: LockHandle, buffer: Vec<u8>) -> Result<(), E>;
    fn sys_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), E>;
    fn sys_get_actor(&mut self) -> Result<ScryptoActor, E>;
    fn sys_generate_uuid(&mut self) -> Result<u128, E>;
    fn sys_emit_log(&mut self, level: Level, message: String) -> Result<(), E>;
}

#[macro_export]
macro_rules! native_methods {
    ($receiver:expr, $type_ident:expr => { $($vis:vis $fn:ident $method_name:ident $s:tt -> $rtn:ty { $fn_ident:expr, $arg:expr })* } ) => {
        $(
            $vis $fn $method_name $s -> $rtn {
                let input = RadixEngineInput::InvokeNativeMethod(
                    $type_ident($fn_ident),
                    $receiver,
                    scrypto::buffer::scrypto_encode(&$arg)
                );
                call_engine(input)
            }
        )+
    };
}
