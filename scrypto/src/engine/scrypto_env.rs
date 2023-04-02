use crate::engine::wasm_api::*;
use radix_engine_interface::api::kernel_modules::auth_api::ClientAuthApi;
use radix_engine_interface::api::{types::*, ClientTransactionRuntimeApi};
use radix_engine_interface::api::{ClientActorApi, ClientObjectApi, ClientSubstateApi};
use radix_engine_interface::api::{ClientEventApi, ClientLoggerApi, LockFlags};
use radix_engine_interface::blueprints::resource::AccessRule;
use radix_engine_interface::crypto::Hash;
use radix_engine_interface::data::scrypto::model::{Address, PackageAddress};
use radix_engine_interface::data::scrypto::*;
use radix_engine_interface::*;
use sbor::rust::prelude::*;
use sbor::*;
use scrypto_schema::{IterableMapSchema, KeyValueStoreSchema};

#[derive(Debug, Sbor)]
pub enum ClientApiError {
    DecodeError(DecodeError),
}

pub struct ScryptoEnv;

impl ClientObjectApi<ClientApiError> for ScryptoEnv {
    fn new_object(
        &mut self,
        blueprint_ident: &str,
        app_states: Vec<Vec<u8>>,
    ) -> Result<ObjectId, ClientApiError> {
        let app_states = scrypto_encode(&app_states).unwrap();

        let bytes = copy_buffer(unsafe {
            new_object(
                blueprint_ident.as_ptr(),
                blueprint_ident.len(),
                app_states.as_ptr(),
                app_states.len(),
            )
        });
        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }

    fn new_key_value_store(
        &mut self,
        schema: KeyValueStoreSchema,
    ) -> Result<KeyValueStoreId, ClientApiError> {
        let schema = scrypto_encode(&schema).unwrap();
        let bytes = copy_buffer(unsafe { new_key_value_store(schema.as_ptr(), schema.len()) });
        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }

    fn get_key_value_store_info(
        &mut self,
        node_id: RENodeId,
    ) -> Result<KeyValueStoreSchema, ClientApiError> {
        let node_id = scrypto_encode(&node_id).unwrap();

        let bytes =
            copy_buffer(unsafe { get_key_value_store_info(node_id.as_ptr(), node_id.len()) });

        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }

    fn globalize(
        &mut self,
        node_id: RENodeId,
        modules: BTreeMap<NodeModuleId, ObjectId>,
    ) -> Result<Address, ClientApiError> {
        let node_id = scrypto_encode(&node_id).unwrap();
        let modules = scrypto_encode(&modules).unwrap();

        let bytes = copy_buffer(unsafe {
            globalize_object(
                node_id.as_ptr(),
                node_id.len(),
                modules.as_ptr(),
                modules.len(),
            )
        });
        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }

    fn globalize_with_address(
        &mut self,
        node_id: RENodeId,
        modules: BTreeMap<NodeModuleId, ObjectId>,
        address: Address,
    ) -> Result<Address, ClientApiError> {
        let node_id = scrypto_encode(&node_id).unwrap();
        let modules = scrypto_encode(&modules).unwrap();
        let address = scrypto_encode(&address).unwrap();

        let bytes = copy_buffer(unsafe {
            globalize_with_address(
                node_id.as_ptr(),
                node_id.len(),
                modules.as_ptr(),
                modules.len(),
                address.as_ptr(),
                address.len(),
            )
        });
        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }

    fn call_method(
        &mut self,
        receiver: &RENodeId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, ClientApiError> {
        self.call_module_method(receiver, NodeModuleId::SELF, method_name, args)
    }

    fn call_module_method(
        &mut self,
        receiver: &RENodeId,
        node_module_id: NodeModuleId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, ClientApiError> {
        let receiver = scrypto_encode(receiver).unwrap();

        let return_data = copy_buffer(unsafe {
            call_method(
                receiver.as_ptr(),
                receiver.len(),
                node_module_id.id(),
                method_name.as_ptr(),
                method_name.len(),
                args.as_ptr(),
                args.len(),
            )
        });

        Ok(return_data)
    }

    fn get_object_info(&mut self, node_id: RENodeId) -> Result<ObjectInfo, ClientApiError> {
        let node_id = scrypto_encode(&node_id).unwrap();

        let bytes = copy_buffer(unsafe { get_object_info(node_id.as_ptr(), node_id.len()) });

        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }

    fn call_function(
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

    fn drop_object(&mut self, node_id: RENodeId) -> Result<(), ClientApiError> {
        let node_id = scrypto_encode(&node_id).unwrap();

        unsafe { drop_object(node_id.as_ptr(), node_id.len()) };

        Ok(())
    }
}

impl ClientSubstateApi<ClientApiError> for ScryptoEnv {
    fn sys_lock_substate(
        &mut self,
        node_id: RENodeId,
        offset: SubstateOffset,
        flags: LockFlags,
    ) -> Result<LockHandle, ClientApiError> {
        let node_id = scrypto_encode(&node_id).unwrap();
        let offset = scrypto_encode(&offset).unwrap();

        let handle = unsafe {
            lock_substate(
                node_id.as_ptr(),
                node_id.len(),
                offset.as_ptr(),
                offset.len(),
                flags.bits(),
            )
        };

        Ok(handle)
    }

    fn sys_read_substate(&mut self, lock_handle: LockHandle) -> Result<Vec<u8>, ClientApiError> {
        let substate = copy_buffer(unsafe { read_substate(lock_handle) });

        Ok(substate)
    }

    fn sys_write_substate(
        &mut self,
        lock_handle: LockHandle,
        buffer: Vec<u8>,
    ) -> Result<(), ClientApiError> {
        unsafe { write_substate(lock_handle, buffer.as_ptr(), buffer.len()) };

        Ok(())
    }

    fn sys_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), ClientApiError> {
        unsafe { drop_lock(lock_handle) };

        Ok(())
    }
}

impl ClientActorApi<ClientApiError> for ScryptoEnv {
    fn get_global_address(&mut self) -> Result<Address, ClientApiError> {
        let global_address = copy_buffer(unsafe { get_global_address() });

        scrypto_decode(&global_address).map_err(ClientApiError::DecodeError)
    }

    fn get_blueprint(&mut self) -> Result<Blueprint, ClientApiError> {
        let actor = copy_buffer(unsafe { get_blueprint() });

        scrypto_decode(&actor).map_err(ClientApiError::DecodeError)
    }
}

impl ClientAuthApi<ClientApiError> for ScryptoEnv {
    fn get_auth_zone(&mut self) -> Result<ObjectId, ClientApiError> {
        let auth_zone = copy_buffer(unsafe { get_auth_zone() });

        scrypto_decode(&auth_zone).map_err(ClientApiError::DecodeError)
    }

    fn assert_access_rule(&mut self, access_rule: AccessRule) -> Result<(), ClientApiError> {
        let access_rule = scrypto_encode(&access_rule).unwrap();

        unsafe { assert_access_rule(access_rule.as_ptr(), access_rule.len()) };

        Ok(())
    }
}

impl ClientEventApi<ClientApiError> for ScryptoEnv {
    fn emit_event(
        &mut self,
        event_name: String,
        event_data: Vec<u8>,
    ) -> Result<(), ClientApiError> {
        unsafe {
            emit_event(
                event_name.as_ptr(),
                event_name.len(),
                event_data.as_ptr(),
                event_data.len(),
            )
        };
        Ok(())
    }
}

impl ClientLoggerApi<ClientApiError> for ScryptoEnv {
    fn log_message(&mut self, level: Level, message: String) -> Result<(), ClientApiError> {
        let level = scrypto_encode(&level).unwrap();
        unsafe { log_message(level.as_ptr(), level.len(), message.as_ptr(), message.len()) }
        Ok(())
    }
}

impl ClientTransactionRuntimeApi<ClientApiError> for ScryptoEnv {
    fn get_transaction_hash(&mut self) -> Result<Hash, ClientApiError> {
        let actor = copy_buffer(unsafe { get_transaction_hash() });

        scrypto_decode(&actor).map_err(ClientApiError::DecodeError)
    }

    fn generate_uuid(&mut self) -> Result<u128, ClientApiError> {
        let actor = copy_buffer(unsafe { generate_uuid() });

        scrypto_decode(&actor).map_err(ClientApiError::DecodeError)
    }
}

#[macro_export]
macro_rules! scrypto_env_native_fn {
    ($($vis:vis $fn:ident $fn_name:ident ($($args:tt)*) -> $rtn:ty { $arg:expr })*) => {
        $(
            $vis $fn $fn_name ($($args)*) -> $rtn {
                let mut env = crate::engine::scrypto_env::ScryptoEnv;
                env.call_native($arg).unwrap()
            }
        )+
    };
}
