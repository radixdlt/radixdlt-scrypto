use crate::engine::wasm_api::*;
use radix_engine_interface::api::types::{
    FnIdentifier, LockHandle, PackageAddress, RENodeId, ScryptoRENode, ScryptoReceiver,
    SerializableInvocation, SubstateOffset,
};
use radix_engine_interface::api::ClientNodeApi;
use radix_engine_interface::api::{ClientActorApi, ClientSubstateApi, Invokable};
use radix_engine_interface::data::{scrypto_decode, scrypto_encode};
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;
use sbor::*;

#[derive(Debug, Categorize, Encode, Decode)]
pub enum ClientApiError {
    DecodeError(DecodeError),
}

pub struct ScryptoEnv;

impl ScryptoEnv {
    // Slightly different from ClientComponentApi::call_method, for the return type.
    // This is to avoid duplicated encoding and decoding.
    pub fn call_method(
        &mut self,
        receiver: ScryptoReceiver,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, ClientApiError> {
        let receiver = scrypto_encode(&receiver).unwrap();

        let return_data = copy_buffer(unsafe {
            call_method(
                receiver.as_ptr(),
                receiver.len(),
                method_name.as_ptr(),
                method_name.len(),
                args.as_ptr(),
                args.len(),
            )
        });

        Ok(return_data)
    }

    // Slightly different from ClientPackageApi::call_function, for the return type.
    // This is to avoid duplicated encoding and decoding.
    pub fn call_function(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, ClientApiError> {
        let package_address = scrypto_encode(&package_address).unwrap();

        let return_data = copy_buffer(unsafe {
            call_function(
                package_address.as_ptr(),
                package_address.len(),
                blueprint_name.as_ptr(),
                blueprint_name.len(),
                function_name.as_ptr(),
                function_name.len(),
                args.as_ptr(),
                args.len(),
            )
        });

        Ok(return_data)
    }

    pub fn call_native<N: SerializableInvocation>(
        &mut self,
        invocation: N,
    ) -> Result<N::Output, ClientApiError> {
        let native_fn = N::native_fn();
        let native_fn = scrypto_encode(&native_fn).unwrap();
        let invocation = scrypto_encode(&invocation).unwrap();

        let return_data = copy_buffer(unsafe {
            call_native(
                native_fn.as_ptr(),
                native_fn.len(),
                invocation.as_ptr(),
                invocation.len(),
            )
        });

        scrypto_decode(&return_data).map_err(ClientApiError::DecodeError)
    }
}

impl<N: SerializableInvocation> Invokable<N, ClientApiError> for ScryptoEnv {
    fn invoke(&mut self, invocation: N) -> Result<N::Output, ClientApiError> {
        self.call_native(invocation)
    }
}

impl ClientNodeApi<ClientApiError> for ScryptoEnv {
    fn sys_create_node(&mut self, node: ScryptoRENode) -> Result<RENodeId, ClientApiError> {
        let node = scrypto_encode(&node).unwrap();

        let node_id = copy_buffer(unsafe { create_node(node.as_ptr(), node.len()) });

        scrypto_decode(&node_id).map_err(ClientApiError::DecodeError)
    }

    fn sys_drop_node(&mut self, node_id: RENodeId) -> Result<(), ClientApiError> {
        let node_id = scrypto_encode(&node_id).unwrap();

        unsafe { drop_node(node_id.as_ptr(), node_id.len()) };

        Ok(())
    }
}

impl ClientSubstateApi<ClientApiError> for ScryptoEnv {
    fn sys_lock_substate(
        &mut self,
        node_id: RENodeId,
        offset: SubstateOffset,
        mutable: bool,
    ) -> Result<LockHandle, ClientApiError> {
        let node_id = scrypto_encode(&node_id).unwrap();
        let offset = scrypto_encode(&offset).unwrap();

        let handle = unsafe {
            lock_substate(
                node_id.as_ptr(),
                node_id.len(),
                offset.as_ptr(),
                offset.len(),
                mutable,
            )
        };

        Ok(handle)
    }

    fn sys_read(&mut self, lock_handle: LockHandle) -> Result<Vec<u8>, ClientApiError> {
        let substate = copy_buffer(unsafe { read_substate(lock_handle) });

        Ok(substate)
    }

    fn sys_write(
        &mut self,
        lock_handle: LockHandle,
        buffer: Vec<u8>,
    ) -> Result<(), ClientApiError> {
        unsafe { write_substate(lock_handle, buffer.as_ptr(), buffer.len()) };

        Ok(())
    }

    fn sys_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), ClientApiError> {
        unsafe { unlock_substate(lock_handle) };

        Ok(())
    }
}

impl ClientActorApi<ClientApiError> for ScryptoEnv {
    fn fn_identifier(&mut self) -> Result<FnIdentifier, ClientApiError> {
        let actor = copy_buffer(unsafe { get_actor() });

        scrypto_decode(&actor).map_err(ClientApiError::DecodeError)
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
