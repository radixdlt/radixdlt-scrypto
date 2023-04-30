use crate::engine::wasm_api::*;
use radix_engine_interface::api::kernel_modules::auth_api::ClientAuthApi;
use radix_engine_interface::api::key_value_store_api::{
    ClientKeyValueStoreApi, KeyValueEntryLockHandle,
};
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::{ClientActorApi, ClientFieldLockApi, ClientObjectApi};
use radix_engine_interface::api::{ClientBlueprintApi, ClientTransactionRuntimeApi};
use radix_engine_interface::api::{ClientEventApi, ClientLoggerApi, LockFlags};
use radix_engine_interface::blueprints::resource::AccessRule;
use radix_engine_interface::crypto::Hash;
use radix_engine_interface::data::scrypto::*;
use radix_engine_interface::types::{Blueprint, GlobalAddress};
use radix_engine_interface::types::{Level, LockHandle, NodeId};
use radix_engine_interface::types::{ObjectInfo, PackageAddress};
use radix_engine_interface::*;
use sbor::rust::prelude::*;
use sbor::*;
use scrypto_schema::{InstanceSchema, KeyValueStoreInfo};

#[derive(Debug, Sbor)]
pub enum ClientApiError {
    DecodeError(DecodeError),
}

pub struct ScryptoEnv;

impl ClientObjectApi<ClientApiError> for ScryptoEnv {
    fn new_simple_object(
        &mut self,
        blueprint_ident: &str,
        object_states: Vec<Vec<u8>>,
    ) -> Result<NodeId, ClientApiError> {
        let object_states = scrypto_encode(&object_states).unwrap();

        let bytes = copy_buffer(unsafe {
            new_object(
                blueprint_ident.as_ptr(),
                blueprint_ident.len(),
                object_states.as_ptr(),
                object_states.len(),
            )
        });
        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }

    fn new_object(
        &mut self,
        _blueprint_ident: &str,
        _schema: Option<InstanceSchema>,
        _fields: Vec<Vec<u8>>,
        _kv_entries: Vec<Vec<(Vec<u8>, Vec<u8>)>>,
    ) -> Result<NodeId, ClientApiError> {
        todo!()
    }

    fn globalize(
        &mut self,
        modules: BTreeMap<ObjectModuleId, NodeId>,
    ) -> Result<GlobalAddress, ClientApiError> {
        let modules = scrypto_encode(&modules).unwrap();

        let bytes = copy_buffer(unsafe { globalize_object(modules.as_ptr(), modules.len()) });
        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }

    fn globalize_with_address(
        &mut self,
        modules: BTreeMap<ObjectModuleId, NodeId>,
        address: GlobalAddress,
    ) -> Result<(), ClientApiError> {
        let modules = scrypto_encode(&modules).unwrap();
        let address = scrypto_encode(&address).unwrap();

        let bytes = copy_buffer(unsafe {
            globalize_with_address(
                modules.as_ptr(),
                modules.len(),
                address.as_ptr(),
                address.len(),
            )
        });
        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }

    fn globalize_with_address_and_create_inner_object(
        &mut self,
        _modules: BTreeMap<ObjectModuleId, NodeId>,
        _address: GlobalAddress,
        _inner_object_blueprint: &str,
        _inner_object_fields: Vec<Vec<u8>>,
    ) -> Result<NodeId, ClientApiError> {
        todo!("Unsupported")
    }

    fn call_method(
        &mut self,
        receiver: &NodeId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, ClientApiError> {
        self.call_module_method(receiver, ObjectModuleId::SELF, method_name, args)
    }

    fn call_module_method(
        &mut self,
        receiver: &NodeId,
        module_id: ObjectModuleId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, ClientApiError> {
        let return_data = copy_buffer(unsafe {
            call_method(
                receiver.as_ref().as_ptr(),
                receiver.as_ref().len(),
                module_id as u8 as u32,
                method_name.as_ptr(),
                method_name.len(),
                args.as_ptr(),
                args.len(),
            )
        });

        Ok(return_data)
    }

    fn get_object_info(&mut self, node_id: &NodeId) -> Result<ObjectInfo, ClientApiError> {
        let bytes = copy_buffer(unsafe {
            get_object_info(node_id.as_ref().as_ptr(), node_id.as_ref().len())
        });

        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }

    fn drop_object(&mut self, _node_id: &NodeId) -> Result<Vec<Vec<u8>>, ClientApiError> {
        // TODO: Remove or implement drop_object interface from scrypto
        //unsafe { drop_object(node_id.as_ref().as_ptr(), node_id.as_ref().len()) };
        todo!("Unsupported")
    }
}

impl ClientKeyValueStoreApi<ClientApiError> for ScryptoEnv {
    fn key_value_store_new(&mut self, schema: KeyValueStoreInfo) -> Result<NodeId, ClientApiError> {
        let schema = scrypto_encode(&schema).unwrap();
        let bytes = copy_buffer(unsafe { new_key_value_store(schema.as_ptr(), schema.len()) });
        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }

    fn key_value_store_get_info(
        &mut self,
        node_id: &NodeId,
    ) -> Result<KeyValueStoreInfo, ClientApiError> {
        let bytes = copy_buffer(unsafe {
            get_key_value_store_info(node_id.as_ref().as_ptr(), node_id.as_ref().len())
        });

        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }

    fn key_value_store_lock_entry(
        &mut self,
        node_id: &NodeId,
        key: &Vec<u8>,
        flags: LockFlags,
    ) -> Result<KeyValueEntryLockHandle, ClientApiError> {
        let handle = unsafe {
            lock_key_value_store_entry(
                node_id.as_ref().as_ptr(),
                node_id.as_ref().len(),
                key.as_ptr(),
                key.len(),
                flags.bits(),
            )
        };

        Ok(handle)
    }

    fn key_value_entry_get(
        &mut self,
        handle: KeyValueEntryLockHandle,
    ) -> Result<Vec<u8>, ClientApiError> {
        let entry = copy_buffer(unsafe { key_value_entry_get(handle) });

        Ok(entry)
    }

    fn key_value_entry_set(
        &mut self,
        handle: KeyValueEntryLockHandle,
        buffer: Vec<u8>,
    ) -> Result<(), ClientApiError> {
        unsafe { key_value_entry_set(handle, buffer.as_ptr(), buffer.len()) };

        Ok(())
    }

    fn key_value_entry_lock_release(
        &mut self,
        handle: KeyValueEntryLockHandle,
    ) -> Result<(), ClientApiError> {
        unsafe { unlock_key_value_entry(handle) };

        Ok(())
    }

    fn key_value_entry_remove(
        &mut self,
        node_id: &NodeId,
        key: &Vec<u8>,
    ) -> Result<Vec<u8>, ClientApiError> {
        let removed = copy_buffer(unsafe {
            key_value_entry_remove(
                node_id.as_ref().as_ptr(),
                node_id.as_ref().len(),
                key.as_ptr(),
                key.len(),
            )
        });
        Ok(removed)
    }
}

impl ClientBlueprintApi<ClientApiError> for ScryptoEnv {
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
}

impl ClientFieldLockApi<ClientApiError> for ScryptoEnv {
    fn field_lock_read(&mut self, lock_handle: LockHandle) -> Result<Vec<u8>, ClientApiError> {
        let substate = copy_buffer(unsafe { read_substate(lock_handle) });

        Ok(substate)
    }

    fn field_lock_write(
        &mut self,
        lock_handle: LockHandle,
        buffer: Vec<u8>,
    ) -> Result<(), ClientApiError> {
        unsafe { write_substate(lock_handle, buffer.as_ptr(), buffer.len()) };

        Ok(())
    }

    fn field_lock_release(&mut self, lock_handle: LockHandle) -> Result<(), ClientApiError> {
        unsafe { drop_lock(lock_handle) };

        Ok(())
    }
}

impl ClientActorApi<ClientApiError> for ScryptoEnv {
    fn lock_field(&mut self, field: u8, flags: LockFlags) -> Result<LockHandle, ClientApiError> {
        let handle = unsafe { lock_field(u32::from(field), flags.bits()) };

        Ok(handle)
    }

    fn lock_outer_object_field(
        &mut self,
        _field: u8,
        _flags: LockFlags,
    ) -> Result<LockHandle, ClientApiError> {
        todo!()
    }

    fn actor_lock_key_value_handle_entry(
        &mut self,
        _kv_handle: u8,
        _key: &Vec<u8>,
        _flags: LockFlags,
    ) -> Result<LockHandle, ClientApiError> {
        todo!()
    }

    fn actor_key_value_entry_remove(&mut self, _key: &Vec<u8>) -> Result<Vec<u8>, ClientApiError> {
        todo!()
    }

    fn get_info(&mut self) -> Result<ObjectInfo, ClientApiError> {
        todo!()
    }

    fn get_global_address(&mut self) -> Result<GlobalAddress, ClientApiError> {
        let global_address = copy_buffer(unsafe { get_global_address() });

        scrypto_decode(&global_address).map_err(ClientApiError::DecodeError)
    }

    fn get_blueprint(&mut self) -> Result<Blueprint, ClientApiError> {
        let actor = copy_buffer(unsafe { get_blueprint() });

        scrypto_decode(&actor).map_err(ClientApiError::DecodeError)
    }
}

impl ClientAuthApi<ClientApiError> for ScryptoEnv {
    fn get_auth_zone(&mut self) -> Result<NodeId, ClientApiError> {
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
