use crate::engine::wasm_api::*;
use radix_engine_common::math::Decimal;
use radix_engine_common::types::GlobalAddressReservation;
use radix_engine_interface::api::field_api::FieldHandle;
use radix_engine_interface::api::key_value_entry_api::KeyValueEntryHandle;
use radix_engine_interface::api::key_value_store_api::KeyValueStoreGenericArgs;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::system_modules::auth_api::ClientAuthApi;
use radix_engine_interface::api::LockFlags;
use radix_engine_interface::api::{ClientActorApi, ClientFieldApi, FieldValue, ObjectHandle};
use radix_engine_interface::api::{ClientBlueprintApi, ClientTransactionRuntimeApi};
use radix_engine_interface::crypto::Hash;
use radix_engine_interface::data::scrypto::*;
use radix_engine_interface::types::PackageAddress;
use radix_engine_interface::types::{BlueprintId, GlobalAddress};
use radix_engine_interface::types::{Level, NodeId, SubstateHandle};
use radix_engine_interface::*;
use sbor::rust::prelude::*;
use sbor::*;

#[derive(Debug, Sbor)]
pub enum ClientApiError {
    DecodeError(DecodeError),
}

pub struct ScryptoVmV1Api;

impl ScryptoVmV1Api {
    pub fn execution_cost_unit_limit(&mut self) -> u32 {
        unsafe { execution_cost_unit_limit() }
    }

    pub fn execution_cost_unit_price(&mut self) -> Decimal {
        let bytes = copy_buffer(unsafe { execution_cost_unit_price() });
        scrypto_decode(&bytes).unwrap()
    }

    pub fn finalization_cost_unit_limit(&mut self) -> u32 {
        unsafe { finalization_cost_unit_limit() }
    }

    pub fn finalization_cost_unit_price(&mut self) -> Decimal {
        let bytes = copy_buffer(unsafe { finalization_cost_unit_price() });
        scrypto_decode(&bytes).unwrap()
    }

    pub fn usd_price(&mut self) -> Decimal {
        let bytes = copy_buffer(unsafe { usd_price() });
        scrypto_decode(&bytes).unwrap()
    }

    pub fn tip_percentage(&mut self) -> u32 {
        unsafe { tip_percentage() }
    }

    pub fn fee_balance(&mut self) -> Decimal {
        let bytes = copy_buffer(unsafe { fee_balance() });
        scrypto_decode(&bytes).unwrap()
    }

    pub fn allocate_global_address(
        &mut self,
        blueprint_id: BlueprintId,
    ) -> (GlobalAddressReservation, GlobalAddress) {
        let blueprint_id = scrypto_encode(&blueprint_id).unwrap();
        let bytes = copy_buffer(unsafe {
            allocate_global_address(blueprint_id.as_ptr(), blueprint_id.len())
        });
        scrypto_decode(&bytes).unwrap()
    }

    pub fn new_simple_object(
        &mut self,
        blueprint_ident: &str,
        object_states: Vec<FieldValue>,
    ) -> NodeId {
        let object_states = scrypto_encode(&object_states).unwrap();

        let bytes = copy_buffer(unsafe {
            new_object(
                blueprint_ident.as_ptr(),
                blueprint_ident.len(),
                object_states.as_ptr(),
                object_states.len(),
            )
        });
        scrypto_decode(&bytes).unwrap()
    }

    pub fn globalize(
        &mut self,
        modules: BTreeMap<ObjectModuleId, NodeId>,
        address_reservation: Option<GlobalAddressReservation>,
    ) -> GlobalAddress {
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
        scrypto_decode(&bytes).unwrap()
    }

    pub fn call_method(&mut self, receiver: &NodeId, method_name: &str, args: Vec<u8>) -> Vec<u8> {
        self.call_method_advanced(receiver, ObjectModuleId::Main, false, method_name, args)
    }

    pub fn call_method_advanced(
        &mut self,
        receiver: &NodeId,
        module_id: ObjectModuleId,
        direct_access: bool,
        method_name: &str,
        args: Vec<u8>,
    ) -> Vec<u8> {
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

        return_data
    }

    pub fn get_blueprint_id(&mut self, node_id: &NodeId) -> BlueprintId {
        let bytes = copy_buffer(unsafe {
            get_blueprint_id(node_id.as_ref().as_ptr(), node_id.as_ref().len())
        });

        scrypto_decode(&bytes).unwrap()
    }

    pub fn get_outer_object(&mut self, node_id: &NodeId) -> GlobalAddress {
        let bytes = copy_buffer(unsafe {
            get_outer_object(node_id.as_ref().as_ptr(), node_id.as_ref().len())
        });

        scrypto_decode(&bytes).unwrap()
    }

    pub fn get_reservation_address(&mut self, node_id: &NodeId) -> GlobalAddress {
        let bytes = copy_buffer(unsafe {
            get_reservation_address(node_id.as_ref().as_ptr(), node_id.as_ref().len())
        });

        scrypto_decode(&bytes).unwrap()
    }

    pub fn drop_object(&mut self, node_id: &NodeId) -> Vec<Vec<u8>> {
        unsafe { drop_object(node_id.as_ref().as_ptr(), node_id.as_ref().len()) };

        // TODO: remove return
        Vec::new()
    }

    pub fn key_value_entry_get(&mut self, handle: KeyValueEntryHandle) -> Vec<u8> {
        let entry = copy_buffer(unsafe { kv_entry_get(handle) });
        entry
    }

    pub fn key_value_entry_set(&mut self, handle: KeyValueEntryHandle, buffer: Vec<u8>) {
        unsafe { kv_entry_set(handle, buffer.as_ptr(), buffer.len()) };
    }

    pub fn key_value_entry_remove(&mut self, handle: KeyValueEntryHandle) -> Vec<u8> {
        let removed = copy_buffer(unsafe { kv_entry_remove(handle) });
        removed
    }

    pub fn key_value_entry_close(&mut self, handle: KeyValueEntryHandle) {
        unsafe { kv_entry_release(handle) };
    }

    pub fn key_value_store_new(&mut self, schema: KeyValueStoreGenericArgs) -> NodeId {
        let schema = scrypto_encode(&schema).unwrap();
        let bytes = copy_buffer(unsafe { kv_store_new(schema.as_ptr(), schema.len()) });
        scrypto_decode(&bytes).unwrap()
    }

    pub fn key_value_store_open_entry(
        &mut self,
        node_id: &NodeId,
        key: &Vec<u8>,
        flags: LockFlags,
    ) -> KeyValueEntryHandle {
        let handle = unsafe {
            kv_store_open_entry(
                node_id.as_ref().as_ptr(),
                node_id.as_ref().len(),
                key.as_ptr(),
                key.len(),
                flags.bits(),
            )
        };

        handle
    }

    pub fn key_value_store_remove_entry(&mut self, node_id: &NodeId, key: &Vec<u8>) -> Vec<u8> {
        let removed = copy_buffer(unsafe {
            kv_store_remove_entry(
                node_id.as_ref().as_ptr(),
                node_id.as_ref().len(),
                key.as_ptr(),
                key.len(),
            )
        });
        removed
    }
}

impl ClientBlueprintApi<ClientApiError> for ScryptoVmV1Api {
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

impl ClientFieldApi<ClientApiError> for ScryptoVmV1Api {
    fn field_read(&mut self, lock_handle: SubstateHandle) -> Result<Vec<u8>, ClientApiError> {
        let substate = copy_buffer(unsafe { field_lock_read(lock_handle) });

        Ok(substate)
    }

    fn field_write(
        &mut self,
        lock_handle: SubstateHandle,
        buffer: Vec<u8>,
    ) -> Result<(), ClientApiError> {
        unsafe { field_lock_write(lock_handle, buffer.as_ptr(), buffer.len()) };

        Ok(())
    }

    fn field_lock(&mut self, _handle: FieldHandle) -> Result<(), ClientApiError> {
        unimplemented!("Not available for Scrypto")
    }

    fn field_close(&mut self, lock_handle: SubstateHandle) -> Result<(), ClientApiError> {
        unsafe { field_lock_release(lock_handle) };

        Ok(())
    }
}

impl ClientActorApi<ClientApiError> for ScryptoVmV1Api {
    fn actor_open_field(
        &mut self,
        object_handle: u32,
        field: u8,
        flags: LockFlags,
    ) -> Result<SubstateHandle, ClientApiError> {
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

    fn actor_get_node_id(&mut self) -> Result<NodeId, ClientApiError> {
        let node_id = copy_buffer(unsafe { get_node_id() });

        scrypto_decode(&node_id).map_err(ClientApiError::DecodeError)
    }

    fn actor_get_global_address(&mut self) -> Result<GlobalAddress, ClientApiError> {
        let global_address = copy_buffer(unsafe { get_global_address() });

        scrypto_decode(&global_address).map_err(ClientApiError::DecodeError)
    }

    fn actor_get_outer_object(&mut self) -> Result<GlobalAddress, ClientApiError> {
        unimplemented!("Not available for Scrypto")
    }

    fn actor_get_blueprint_id(&mut self) -> Result<BlueprintId, ClientApiError> {
        let actor = copy_buffer(unsafe { get_blueprint() });

        scrypto_decode(&actor).map_err(ClientApiError::DecodeError)
    }

    fn actor_call_module(
        &mut self,
        module_id: ObjectModuleId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, ClientApiError> {
        let return_data = copy_buffer(unsafe {
            actor_call_module_method(
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

impl ClientAuthApi<ClientApiError> for ScryptoVmV1Api {
    fn get_auth_zone(&mut self) -> Result<NodeId, ClientApiError> {
        let auth_zone = copy_buffer(unsafe { get_auth_zone() });

        scrypto_decode(&auth_zone).map_err(ClientApiError::DecodeError)
    }
}

impl ClientTransactionRuntimeApi<ClientApiError> for ScryptoVmV1Api {
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
