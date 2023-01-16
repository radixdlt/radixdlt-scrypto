use crate::engine::wasm_api::*;
use radix_engine_interface::api::types::{
    FnIdentifier, LockHandle, RENodeId, ScryptoRENode, ScryptoReceiver, SubstateOffset,
};
use radix_engine_interface::api::{ActorApi, ComponentApi, EngineApi, Invokable};
use radix_engine_interface::data::ScryptoValue;
use radix_engine_interface::wasm::*;
use sbor::rust::fmt::Debug;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use sbor::*;

#[derive(Debug, Categorize, Encode, Decode)]
pub struct EngineApiError;

pub struct ScryptoEnv;

impl<N: SerializableInvocation> Invokable<N, EngineApiError> for ScryptoEnv {
    fn invoke(&mut self, input: N) -> Result<N::Output, EngineApiError> {
        let rtn = call_engine(RadixEngineInput::Invoke(input.into()));
        Ok(rtn)
    }
}

impl EngineApi<EngineApiError> for ScryptoEnv {
    fn sys_create_node(&mut self, node: ScryptoRENode) -> Result<RENodeId, EngineApiError> {
        let rtn = call_engine(RadixEngineInput::CreateNode(node));
        Ok(rtn)
    }

    fn sys_drop_node(&mut self, node_id: RENodeId) -> Result<(), EngineApiError> {
        let rtn = call_engine(RadixEngineInput::DropNode(node_id));
        Ok(rtn)
    }

    fn sys_get_visible_nodes(&mut self) -> Result<Vec<RENodeId>, EngineApiError> {
        let rtn = call_engine(RadixEngineInput::GetVisibleNodeIds());
        Ok(rtn)
    }

    fn sys_lock_substate(
        &mut self,
        node_id: RENodeId,
        offset: SubstateOffset,
        mutable: bool,
    ) -> Result<LockHandle, EngineApiError> {
        let rtn = call_engine(RadixEngineInput::LockSubstate(node_id, offset, mutable));
        Ok(rtn)
    }

    fn sys_read(&mut self, lock_handle: LockHandle) -> Result<Vec<u8>, EngineApiError> {
        let rtn = call_engine_to_raw(RadixEngineInput::Read(lock_handle));
        Ok(rtn)
    }

    fn sys_write(
        &mut self,
        lock_handle: LockHandle,
        buffer: Vec<u8>,
    ) -> Result<(), EngineApiError> {
        let rtn = call_engine(RadixEngineInput::Write(lock_handle, buffer));
        Ok(rtn)
    }

    fn sys_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), EngineApiError> {
        let rtn = call_engine(RadixEngineInput::DropLock(lock_handle));
        Ok(rtn)
    }
}

impl ComponentApi<EngineApiError> for ScryptoEnv {
    fn invoke_method(
        &mut self,
        receiver: ScryptoReceiver,
        method_name: &str,
        args: &ScryptoValue,
    ) -> Result<ScryptoValue, EngineApiError> {
        let rtn = call_engine(RadixEngineInput::InvokeMethod(
            receiver,
            method_name.to_string(),
            args.clone(),
        ));
        Ok(rtn)
    }
}

impl ActorApi<EngineApiError> for ScryptoEnv {
    fn fn_identifier(&mut self) -> Result<FnIdentifier, EngineApiError> {
        let rtn = call_engine(RadixEngineInput::GetActor());
        Ok(rtn)
    }
}

#[macro_export]
macro_rules! scrypto_env_native_fn {
    ($($vis:vis $fn:ident $fn_name:ident ($($args:tt)*) -> $rtn:ty { $arg:expr })*) => {
        $(
            $vis $fn $fn_name ($($args)*) -> $rtn {
                let mut env = crate::engine::scrypto_env::ScryptoEnv;
                radix_engine_interface::api::Invokable::invoke(&mut env, $arg).unwrap()
            }
        )+
    };
}
