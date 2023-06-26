use crate::engine::wasm_api::*;
use radix_engine_common::math::Decimal;
use radix_engine_common::types::GlobalAddressReservation;
use radix_engine_interface::api::key_value_entry_api::{
    ClientKeyValueEntryApi, KeyValueEntryHandle,
};
use radix_engine_interface::api::key_value_store_api::ClientKeyValueStoreApi;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::system_modules::auth_api::ClientAuthApi;
use radix_engine_interface::api::{
    ClientActorApi, ClientCostingApi, ClientFieldLockApi, ClientObjectApi, ObjectHandle,
};
use radix_engine_interface::api::{ClientBlueprintApi, ClientTransactionRuntimeApi};
use radix_engine_interface::api::{KVEntry, LockFlags};
use radix_engine_interface::blueprints::resource::AccessRule;
use radix_engine_interface::crypto::Hash;
use radix_engine_interface::data::scrypto::*;
use radix_engine_interface::types::{BlueprintId, GlobalAddress};
use radix_engine_interface::types::{Level, LockHandle, NodeId};
use radix_engine_interface::types::{ObjectInfo, PackageAddress};
use radix_engine_interface::*;
use sbor::rust::prelude::*;
use sbor::*;
use scrypto_schema::{InstanceSchema, KeyValueStoreSchema};

#[derive(Debug, Sbor)]
pub enum ClientApiError {
    DecodeError(DecodeError),
}

pub struct ScryptoEnv;

impl ClientCostingApi<ClientApiError> for ScryptoEnv {
    fn consume_cost_units(
        &mut self,
        _costing_entry: types::ClientCostingEntry,
    ) -> Result<(), ClientApiError> {
        unimplemented!("Not exposed to scrypto")
    }

    fn credit_cost_units(
        &mut self,
        _vault_id: NodeId,
        _locked_fee: blueprints::resource::LiquidFungibleResource,
        _contingent: bool,
    ) -> Result<blueprints::resource::LiquidFungibleResource, ClientApiError> {
        unimplemented!("Not exposed to scrypto")
    }

    fn cost_unit_limit(&mut self) -> Result<u32, ClientApiError> {
        Ok(unsafe { cost_unit_limit() })
    }

    fn cost_unit_price(&mut self) -> Result<math::Decimal, ClientApiError> {
        let bytes = copy_buffer(unsafe { cost_unit_price() });
        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }

    fn usd_price(&mut self) -> Result<Decimal, ClientApiError> {
        unimplemented!("Not exposed to scrypto")
    }

    fn max_per_function_royalty_in_xrd(&mut self) -> Result<Decimal, ClientApiError> {
        unimplemented!("Not exposed to scrypto")
    }

    fn tip_percentage(&mut self) -> Result<u32, ClientApiError> {
        Ok(unsafe { tip_percentage() })
    }

    fn fee_balance(&mut self) -> Result<math::Decimal, ClientApiError> {
        let bytes = copy_buffer(unsafe { fee_balance() });
        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }
}

// FIXME: finalize API

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
        _features: Vec<&str>,
        _schema: Option<InstanceSchema>,
        _fields: Vec<Vec<u8>>,
        _kv_entries: BTreeMap<u8, BTreeMap<Vec<u8>, KVEntry>>,
    ) -> Result<NodeId, ClientApiError> {
        unimplemented!("Not available for Scrypto")
    }

    fn allocate_global_address(
        &mut self,
        blueprint_id: BlueprintId,
    ) -> Result<(GlobalAddressReservation, GlobalAddress), ClientApiError> {
        let blueprint_id = scrypto_encode(&blueprint_id).unwrap();
        let bytes = copy_buffer(unsafe {
            allocate_global_address(blueprint_id.as_ptr(), blueprint_id.len())
        });
        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }

    fn globalize(
        &mut self,
        modules: BTreeMap<ObjectModuleId, NodeId>,
        address_reservation: Option<GlobalAddressReservation>,
    ) -> Result<GlobalAddress, ClientApiError> {
        let modules = scrypto_encode(&modules).unwrap();
        let address_reservation = scrypto_encode(&address_reservation).unwrap();

        let bytes = copy_buffer(unsafe {
            globalize(
                modules.as_ptr(),
                modules.len(),
                address_reservation.as_ptr(),
                address_reservation.len(),
            )
        });
        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }

    fn globalize_with_address_and_create_inner_object(
        &mut self,
        _modules: BTreeMap<ObjectModuleId, NodeId>,
        _address_reservation: GlobalAddressReservation,
        _inner_object_blueprint: &str,
        _inner_object_fields: Vec<Vec<u8>>,
    ) -> Result<(GlobalAddress, NodeId), ClientApiError> {
        unimplemented!("Not available for Scrypto")
    }

    fn call_method_advanced(
        &mut self,
        receiver: &NodeId,
        direct_access: bool,
        module_id: ObjectModuleId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, ClientApiError> {
        let return_data = copy_buffer(unsafe {
            call_method(
                receiver.as_ref().as_ptr(),
                receiver.as_ref().len(),
                if direct_access { 1 } else { 0 },
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

    fn get_reservation_address(
        &mut self,
        _node_id: &NodeId,
    ) -> Result<GlobalAddress, ClientApiError> {
        // FIXME: Implement this for Scrypto
        todo!()
    }

    fn drop_object(&mut self, node_id: &NodeId) -> Result<Vec<Vec<u8>>, ClientApiError> {
        unsafe { drop_object(node_id.as_ref().as_ptr(), node_id.as_ref().len()) };

        // TODO: remove return
        Ok(Vec::new())
    }

    fn allocate_virtual_global_address(
        &mut self,
        _blueprint_id: BlueprintId,
        _global_address: GlobalAddress,
    ) -> Result<GlobalAddressReservation, ClientApiError> {
        unimplemented!()
    }
}

impl ClientKeyValueEntryApi<ClientApiError> for ScryptoEnv {
    fn key_value_entry_get(
        &mut self,
        handle: KeyValueEntryHandle,
    ) -> Result<Vec<u8>, ClientApiError> {
        let entry = copy_buffer(unsafe { kv_entry_get(handle) });

        Ok(entry)
    }

    fn key_value_entry_set(
        &mut self,
        handle: KeyValueEntryHandle,
        buffer: Vec<u8>,
    ) -> Result<(), ClientApiError> {
        unsafe { kv_entry_set(handle, buffer.as_ptr(), buffer.len()) };

        Ok(())
    }

    fn key_value_entry_remove(
        &mut self,
        _handle: KeyValueEntryHandle,
    ) -> Result<Vec<u8>, ClientApiError> {
        unimplemented!("Not available for Scrypto")
    }

    fn key_value_entry_freeze(
        &mut self,
        _handle: KeyValueEntryHandle,
    ) -> Result<(), ClientApiError> {
        unimplemented!("Not available for Scrypto")
    }

    fn key_value_entry_release(
        &mut self,
        handle: KeyValueEntryHandle,
    ) -> Result<(), ClientApiError> {
        unsafe { kv_entry_release(handle) };

        Ok(())
    }
}

impl ClientKeyValueStoreApi<ClientApiError> for ScryptoEnv {
    fn key_value_store_new(
        &mut self,
        schema: KeyValueStoreSchema,
    ) -> Result<NodeId, ClientApiError> {
        let schema = scrypto_encode(&schema).unwrap();
        let bytes = copy_buffer(unsafe { kv_store_new(schema.as_ptr(), schema.len()) });
        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }

    fn key_value_store_get_info(
        &mut self,
        node_id: &NodeId,
    ) -> Result<KeyValueStoreSchema, ClientApiError> {
        let bytes = copy_buffer(unsafe {
            kv_store_get_info(node_id.as_ref().as_ptr(), node_id.as_ref().len())
        });

        scrypto_decode(&bytes).map_err(ClientApiError::DecodeError)
    }

    fn key_value_store_open_entry(
        &mut self,
        node_id: &NodeId,
        key: &Vec<u8>,
        flags: LockFlags,
    ) -> Result<KeyValueEntryHandle, ClientApiError> {
        let handle = unsafe {
            kv_store_open_entry(
                node_id.as_ref().as_ptr(),
                node_id.as_ref().len(),
                key.as_ptr(),
                key.len(),
                flags.bits(),
            )
        };

        Ok(handle)
    }

    fn key_value_store_remove_entry(
        &mut self,
        node_id: &NodeId,
        key: &Vec<u8>,
    ) -> Result<Vec<u8>, ClientApiError> {
        let removed = copy_buffer(unsafe {
            kv_store_remove_entry(
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
        let substate = copy_buffer(unsafe { field_lock_read(lock_handle) });

        Ok(substate)
    }

    fn field_lock_write(
        &mut self,
        lock_handle: LockHandle,
        buffer: Vec<u8>,
    ) -> Result<(), ClientApiError> {
        unsafe { field_lock_write(lock_handle, buffer.as_ptr(), buffer.len()) };

        Ok(())
    }

    fn field_lock_release(&mut self, lock_handle: LockHandle) -> Result<(), ClientApiError> {
        unsafe { field_lock_release(lock_handle) };

        Ok(())
    }
}

impl ClientActorApi<ClientApiError> for ScryptoEnv {
    fn actor_open_field(
        &mut self,
        object_handle: u32,
        field: u8,
        flags: LockFlags,
    ) -> Result<LockHandle, ClientApiError> {
        let handle = unsafe { actor_open_field(object_handle, u32::from(field), flags.bits()) };

        Ok(handle)
    }

    fn actor_is_feature_enabled(
        &mut self,
        _: ObjectHandle,
        _feature: &str,
    ) -> Result<bool, ClientApiError> {
        unimplemented!("Not available for Scrypto")
    }

    fn actor_get_info(&mut self) -> Result<ObjectInfo, ClientApiError> {
        unimplemented!("Not available for Scrypto")
    }

    fn actor_get_node_id(&mut self) -> Result<NodeId, ClientApiError> {
        let node_id = copy_buffer(unsafe { get_node_id() });

        scrypto_decode(&node_id).map_err(ClientApiError::DecodeError)
    }

    fn actor_get_global_address(&mut self) -> Result<GlobalAddress, ClientApiError> {
        let global_address = copy_buffer(unsafe { get_global_address() });

        scrypto_decode(&global_address).map_err(ClientApiError::DecodeError)
    }

    fn actor_get_blueprint(&mut self) -> Result<BlueprintId, ClientApiError> {
        let actor = copy_buffer(unsafe { get_blueprint() });

        scrypto_decode(&actor).map_err(ClientApiError::DecodeError)
    }

    fn actor_call_module_method(
        &mut self,
        object_handle: ObjectHandle,
        module_id: ObjectModuleId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, ClientApiError> {
        let return_data = copy_buffer(unsafe {
            actor_call_module_method(
                object_handle,
                module_id as u8 as u32,
                method_name.as_ptr(),
                method_name.len(),
                args.as_ptr(),
                args.len(),
            )
        });

        Ok(return_data)
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

impl ClientTransactionRuntimeApi<ClientApiError> for ScryptoEnv {
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

    fn emit_log(&mut self, level: Level, message: String) -> Result<(), ClientApiError> {
        let level = scrypto_encode(&level).unwrap();
        unsafe { emit_log(level.as_ptr(), level.len(), message.as_ptr(), message.len()) }
        Ok(())
    }

    fn get_transaction_hash(&mut self) -> Result<Hash, ClientApiError> {
        let actor = copy_buffer(unsafe { get_transaction_hash() });

        scrypto_decode(&actor).map_err(ClientApiError::DecodeError)
    }

    fn generate_ruid(&mut self) -> Result<[u8; 32], ClientApiError> {
        let actor = copy_buffer(unsafe { generate_ruid() });

        scrypto_decode(&actor).map_err(ClientApiError::DecodeError)
    }

    fn panic(&mut self, message: String) -> Result<(), ClientApiError> {
        unsafe {
            panic(message.as_ptr(), message.len());
        };
        Ok(())
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
