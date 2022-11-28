use radix_engine_interface::api::api::{EngineApi, SysNativeInvokable};
use radix_engine_interface::api::types::{
    Level, LockHandle, RENodeId, ScryptoActor, ScryptoFunctionIdent, ScryptoMethodIdent,
    ScryptoRENode, SubstateOffset,
};
use radix_engine_interface::crypto::Hash;
use radix_engine_interface::data::ScryptoDecode;
use radix_engine_interface::wasm::*;
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;

#[cfg(target_arch = "wasm32")]
extern "C" {
    pub fn radix_engine(input: *mut u8) -> *mut u8;
}

/// Utility function for making a radix engine call.
#[cfg(target_arch = "wasm32")]
pub fn call_engine<V: ScryptoDecode>(input: RadixEngineInput) -> V {
    use crate::buffer::{scrypto_decode_from_buffer, *};

    unsafe {
        let input_ptr = scrypto_encode_to_buffer(&input).unwrap();
        let output_ptr = radix_engine(input_ptr);
        scrypto_decode_from_buffer::<V>(output_ptr).unwrap()
    }
}

/// Utility function for making a radix engine call.
#[cfg(target_arch = "wasm32")]
pub fn call_engine_to_raw(input: RadixEngineInput) -> Vec<u8> {
    use crate::buffer::{scrypto_buffer_to_vec, *};

    unsafe {
        let input_ptr = scrypto_encode_to_buffer(&input).unwrap();
        let output_ptr = radix_engine(input_ptr);
        scrypto_buffer_to_vec(output_ptr)
    }
}

/// Utility function for making a radix engine call.
#[cfg(not(target_arch = "wasm32"))]
pub fn call_engine<V: ScryptoDecode>(_input: RadixEngineInput) -> V {
    todo!()
}
/// Utility function for making a radix engine call.
#[cfg(not(target_arch = "wasm32"))]
pub fn call_engine_to_raw(_input: RadixEngineInput) -> Vec<u8> {
    todo!()
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct SyscallError;

pub struct ScryptoEnv;

impl<N: ScryptoNativeInvocation> SysNativeInvokable<N, SyscallError> for ScryptoEnv {
    fn sys_invoke(&mut self, input: N) -> Result<N::Output, SyscallError> {
        let rtn = call_engine(RadixEngineInput::InvokeNativeFn(input.into()));
        Ok(rtn)
    }
}

impl EngineApi<SyscallError> for ScryptoEnv {
    fn sys_invoke_scrypto_function(
        &mut self,
        fn_ident: ScryptoFunctionIdent,
        args: Vec<u8>, // TODO: Update to any
    ) -> Result<Vec<u8>, SyscallError> {
        let rtn = call_engine_to_raw(RadixEngineInput::InvokeScryptoFunction(fn_ident, args));
        Ok(rtn)
    }

    fn sys_invoke_scrypto_method(
        &mut self,
        method_ident: ScryptoMethodIdent,
        args: Vec<u8>, // TODO: Update to any
    ) -> Result<Vec<u8>, SyscallError> {
        let rtn = call_engine_to_raw(RadixEngineInput::InvokeScryptoMethod(method_ident, args));
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

    fn sys_read(&mut self, lock_handle: LockHandle) -> Result<Vec<u8>, SyscallError> {
        let rtn = call_engine_to_raw(RadixEngineInput::Read(lock_handle));
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

    fn sys_get_transaction_hash(&mut self) -> Result<Hash, SyscallError> {
        let rtn = call_engine(RadixEngineInput::GetTransactionHash());
        Ok(rtn)
    }

    fn sys_emit_log(&mut self, level: Level, message: String) -> Result<(), SyscallError> {
        let rtn = call_engine(RadixEngineInput::EmitLog(level, message));
        Ok(rtn)
    }
}

#[macro_export]
macro_rules! scrypto_env_native_fn {
    ($($vis:vis $fn:ident $fn_name:ident ($($args:tt)*) -> $rtn:ty { $arg:expr })*) => {
        $(
            $vis $fn $fn_name ($($args)*) -> $rtn {
                let mut env = crate::engine::scrypto_env::ScryptoEnv;
                radix_engine_interface::api::api::SysNativeInvokable::sys_invoke(&mut env, $arg).unwrap()
            }
        )+
    };
}

#[macro_export]
macro_rules! sys_env_native_fn {
    ($vis:vis $fn:ident $fn_name:ident ($($args:tt)+) -> $rtn:ty { $invocation:ident { $($invocation_args:tt)* } }) => {
        $vis $fn $fn_name<Y, E>($($args)*, env: &mut Y) -> Result<$rtn, E>
        where
            Y: radix_engine_interface::api::api::SysNativeInvokable<$invocation, E>,
            E: sbor::rust::fmt::Debug + TypeId<radix_engine_interface::data::ScryptoCustomTypeId> + radix_engine_interface::data::ScryptoDecode,
        {
            radix_engine_interface::api::api::SysNativeInvokable::sys_invoke(env, $invocation { $($invocation_args)* })
        }
    };

    ($vis:vis $fn:ident $fn_name:ident () -> $rtn:ty { $invocation:ident { $($invocation_args:tt)* } }) => {
        $vis $fn $fn_name<Y, E>(env: &mut Y) -> Result<$rtn, E>
        where
            Y: radix_engine_interface::api::api::SysNativeInvokable<$invocation, E>,
            E: sbor::rust::fmt::Debug + TypeId<radix_engine_interface::data::ScryptoCustomTypeId> + radix_engine_interface::data::ScryptoDecode,
        {
            radix_engine_interface::api::api::SysNativeInvokable::sys_invoke(env, $invocation { $($invocation_args)* })
        }
    };
}
