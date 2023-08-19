use crate::engine::wasm_api::*;
use radix_engine_common::math::Decimal;
use radix_engine_common::types::GlobalAddressReservation;
use radix_engine_interface::api::key_value_entry_api::KeyValueEntryHandle;
use radix_engine_interface::api::key_value_store_api::KeyValueStoreGenericArgs;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::FieldValue;
use radix_engine_interface::api::LockFlags;
use radix_engine_interface::crypto::Hash;
use radix_engine_interface::data::scrypto::*;
use radix_engine_interface::types::PackageAddress;
use radix_engine_interface::types::{BlueprintId, GlobalAddress};
use radix_engine_interface::types::{Level, NodeId, SubstateHandle};
use radix_engine_interface::*;
use sbor::rust::prelude::*;

pub struct ScryptoVmV1Api;

impl ScryptoVmV1Api {
    // Costing
    pub fn execution_cost_unit_limit(&mut self) -> u32 {
        unsafe { costing::execution_cost_unit_limit() }
    }

    pub fn execution_cost_unit_price(&mut self) -> Decimal {
        let bytes = copy_buffer(unsafe { costing::execution_cost_unit_price() });
        scrypto_decode(&bytes).unwrap()
    }

    pub fn finalization_cost_unit_limit(&mut self) -> u32 {
        unsafe { costing::finalization_cost_unit_limit() }
    }

    pub fn finalization_cost_unit_price(&mut self) -> Decimal {
        let bytes = copy_buffer(unsafe { costing::finalization_cost_unit_price() });
        scrypto_decode(&bytes).unwrap()
    }

    pub fn usd_price(&mut self) -> Decimal {
        let bytes = copy_buffer(unsafe { costing::usd_price() });
        scrypto_decode(&bytes).unwrap()
    }

    pub fn tip_percentage(&mut self) -> u32 {
        unsafe { costing::tip_percentage() }
    }

    pub fn fee_balance(&mut self) -> Decimal {
        let bytes = copy_buffer(unsafe { costing::fee_balance() });
        scrypto_decode(&bytes).unwrap()
    }

    pub fn allocate_global_address(
        &mut self,
        blueprint_id: BlueprintId,
    ) -> (GlobalAddressReservation, GlobalAddress) {
        let blueprint_id = scrypto_encode(&blueprint_id).unwrap();
        let bytes = copy_buffer(unsafe {
            object::allocate_global_address(blueprint_id.as_ptr(), blueprint_id.len())
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
            object::new_object(
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
            object::globalize(
                modules.as_ptr(),
                modules.len(),
                address_reservation.as_ptr(),
                address_reservation.len(),
            )
        });
        scrypto_decode(&bytes).unwrap()
    }

    pub fn get_blueprint_id(&mut self, node_id: &NodeId) -> BlueprintId {
        let bytes = copy_buffer(unsafe {
            object::get_blueprint_id(node_id.as_ref().as_ptr(), node_id.as_ref().len())
        });

        scrypto_decode(&bytes).unwrap()
    }

    pub fn get_outer_object(&mut self, node_id: &NodeId) -> GlobalAddress {
        let bytes = copy_buffer(unsafe {
            object::get_outer_object(node_id.as_ref().as_ptr(), node_id.as_ref().len())
        });

        scrypto_decode(&bytes).unwrap()
    }

    pub fn get_reservation_address(&mut self, node_id: &NodeId) -> GlobalAddress {
        let bytes = copy_buffer(unsafe {
            object::get_reservation_address(node_id.as_ref().as_ptr(), node_id.as_ref().len())
        });

        scrypto_decode(&bytes).unwrap()
    }

    pub fn drop_object(&mut self, node_id: &NodeId) -> Vec<Vec<u8>> {
        unsafe { object::drop_object(node_id.as_ref().as_ptr(), node_id.as_ref().len()) };

        // TODO: remove return
        Vec::new()
    }

    pub fn key_value_entry_get(&mut self, handle: KeyValueEntryHandle) -> Vec<u8> {
        copy_buffer(unsafe { kv_entry::kv_entry_get(handle) })
    }

    pub fn key_value_entry_set(&mut self, handle: KeyValueEntryHandle, buffer: Vec<u8>) {
        unsafe { kv_entry::kv_entry_set(handle, buffer.as_ptr(), buffer.len()) };
    }

    pub fn key_value_entry_remove(&mut self, handle: KeyValueEntryHandle) -> Vec<u8> {
        copy_buffer(unsafe { kv_entry::kv_entry_remove(handle) })
    }

    pub fn key_value_entry_close(&mut self, handle: KeyValueEntryHandle) {
        unsafe { kv_entry::kv_entry_close(handle) };
    }

    pub fn key_value_store_new(&mut self, schema: KeyValueStoreGenericArgs) -> NodeId {
        let schema = scrypto_encode(&schema).unwrap();
        let bytes = copy_buffer(unsafe { kv_store::kv_store_new(schema.as_ptr(), schema.len()) });
        scrypto_decode(&bytes).unwrap()
    }

    pub fn key_value_store_open_entry(
        &mut self,
        node_id: &NodeId,
        key: &Vec<u8>,
        flags: LockFlags,
    ) -> KeyValueEntryHandle {
        let handle = unsafe {
            kv_store::kv_store_open_entry(
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
            kv_store::kv_store_remove_entry(
                node_id.as_ref().as_ptr(),
                node_id.as_ref().len(),
                key.as_ptr(),
                key.len(),
            )
        });
        removed
    }

    pub fn call_method(&mut self, receiver: &NodeId, method_name: &str, args: Vec<u8>) -> Vec<u8> {
        copy_buffer(unsafe {
            invocation::call_method(
                receiver.as_ref().as_ptr(),
                receiver.as_ref().len(),
                method_name.as_ptr(),
                method_name.len(),
                args.as_ptr(),
                args.len(),
            )
        })
    }

    pub fn call_module_method(
        &mut self,
        receiver: &NodeId,
        module_id: ObjectModuleId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Vec<u8> {
        copy_buffer(unsafe {
            invocation::call_module_method(
                receiver.as_ref().as_ptr(),
                receiver.as_ref().len(),
                module_id as u8 as u32,
                method_name.as_ptr(),
                method_name.len(),
                args.as_ptr(),
                args.len(),
            )
        })
    }

    pub fn call_direct_method(
        &mut self,
        receiver: &NodeId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Vec<u8> {
        copy_buffer(unsafe {
            invocation::call_direct_method(
                receiver.as_ref().as_ptr(),
                receiver.as_ref().len(),
                method_name.as_ptr(),
                method_name.len(),
                args.as_ptr(),
                args.len(),
            )
        })
    }

    pub fn call_function(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: Vec<u8>,
    ) -> Vec<u8> {
        let package_address = scrypto_encode(&package_address).unwrap();

        copy_buffer(unsafe {
            invocation::call_function(
                package_address.as_ptr(),
                package_address.len(),
                blueprint_name.as_ptr(),
                blueprint_name.len(),
                function_name.as_ptr(),
                function_name.len(),
                args.as_ptr(),
                args.len(),
            )
        })
    }

    pub fn field_read(&mut self, lock_handle: SubstateHandle) -> Vec<u8> {
        copy_buffer(unsafe { field_entry::field_entry_read(lock_handle) })
    }

    pub fn field_write(&mut self, lock_handle: SubstateHandle, buffer: Vec<u8>) {
        unsafe { field_entry::field_entry_write(lock_handle, buffer.as_ptr(), buffer.len()) };
    }

    pub fn field_close(&mut self, lock_handle: SubstateHandle) {
        unsafe { field_entry::field_entry_close(lock_handle) };
    }

    pub fn actor_open_field(
        &mut self,
        object_handle: u32,
        field: u8,
        flags: LockFlags,
    ) -> SubstateHandle {
        let handle =
            unsafe { actor::actor_open_field(object_handle, u32::from(field), flags.bits()) };
        handle
    }

    pub fn actor_get_node_id(&mut self) -> NodeId {
        let node_id = copy_buffer(unsafe { actor::get_node_id() });

        scrypto_decode(&node_id).unwrap()
    }

    pub fn actor_get_global_address(&mut self) -> GlobalAddress {
        let global_address = copy_buffer(unsafe { actor::get_global_address() });

        scrypto_decode(&global_address).unwrap()
    }

    pub fn actor_get_blueprint_id(&mut self) -> BlueprintId {
        let blueprint_id = copy_buffer(unsafe { actor::get_blueprint() });

        scrypto_decode(&blueprint_id).unwrap()
    }

    pub fn actor_call_module(
        &mut self,
        module_id: ObjectModuleId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Vec<u8> {
        let return_data = copy_buffer(unsafe {
            actor::actor_call_module_method(
                module_id as u8 as u32,
                method_name.as_ptr(),
                method_name.len(),
                args.as_ptr(),
                args.len(),
            )
        });

        return_data
    }

    pub fn actor_get_auth_zone(&mut self) -> NodeId {
        let auth_zone = copy_buffer(unsafe { actor::get_auth_zone() });

        scrypto_decode(&auth_zone).unwrap()
    }

    pub fn actor_emit_event(&mut self, event_name: String, event_data: Vec<u8>) {
        unsafe {
            actor::emit_event(
                event_name.as_ptr(),
                event_name.len(),
                event_data.as_ptr(),
                event_data.len(),
            )
        };
    }

    pub fn emit_log(&mut self, level: Level, message: String) {
        let level = scrypto_encode(&level).unwrap();
        unsafe { system::emit_log(level.as_ptr(), level.len(), message.as_ptr(), message.len()) }
    }

    pub fn get_transaction_hash(&mut self) -> Hash {
        let actor = copy_buffer(unsafe { system::get_transaction_hash() });

        scrypto_decode(&actor).unwrap()
    }

    pub fn generate_ruid(&mut self) -> [u8; 32] {
        let actor = copy_buffer(unsafe { system::generate_ruid() });

        scrypto_decode(&actor).unwrap()
    }

    pub fn panic(&mut self, message: String) {
        unsafe {
            system::panic(message.as_ptr(), message.len());
        };
    }
}
