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
        Ok(call_engine_wasm_api::<Invoke>(input.into()))
    }
}

impl EngineApi<EngineApiError> for ScryptoEnv {
    fn sys_create_node(&mut self, node: ScryptoRENode) -> Result<RENodeId, EngineApiError> {
        Ok(call_engine_wasm_api::<CreateNode>(node))
    }

    fn sys_drop_node(&mut self, node_id: RENodeId) -> Result<(), EngineApiError> {
        Ok(call_engine_wasm_api::<DropNode>(node_id))
    }

    fn sys_get_visible_nodes(&mut self) -> Result<Vec<RENodeId>, EngineApiError> {
        Ok(call_engine_wasm_api::<GetVisibleNodeIds>(()))
    }

    fn sys_lock_substate(
        &mut self,
        node_id: RENodeId,
        offset: SubstateOffset,
        mutable: bool,
    ) -> Result<LockHandle, EngineApiError> {
        Ok(call_engine_wasm_api::<LockSubstate>((
            node_id, offset, mutable,
        )))
    }

    fn sys_read(&mut self, lock_handle: LockHandle) -> Result<Vec<u8>, EngineApiError> {
        Ok(call_engine_wasm_api::<Read>(lock_handle))
    }

    fn sys_write(
        &mut self,
        lock_handle: LockHandle,
        buffer: Vec<u8>,
    ) -> Result<(), EngineApiError> {
        Ok(call_engine_wasm_api::<Write>((lock_handle, buffer)))
    }

    fn sys_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), EngineApiError> {
        Ok(call_engine_wasm_api::<DropLock>(lock_handle))
    }
}

impl ComponentApi<EngineApiError> for ScryptoEnv {
    fn invoke_method(
        &mut self,
        receiver: ScryptoReceiver,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, EngineApiError> {
        Ok(call_engine_wasm_api::<InvokeMethod>((
            receiver,
            method_name.to_string(),
            args,
        )))
    }
}

impl ActorApi<EngineApiError> for ScryptoEnv {
    fn fn_identifier(&mut self) -> Result<FnIdentifier, EngineApiError> {
        Ok(call_engine_wasm_api::<GetActor>(()))
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
