use crate::engine::wasm_api::*;
use radix_engine_common::math::Decimal;
use radix_engine_common::types::GlobalAddressReservation;
use radix_engine_interface::api::key_value_entry_api::KeyValueEntryHandle;
use radix_engine_interface::api::key_value_store_api::KeyValueStoreGenericArgs;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::{ActorRefHandle, FieldValue};
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
        unsafe { costing::costing_execution_cost_unit_limit() }
    }

    pub fn execution_cost_unit_price(&mut self) -> Decimal {
        let bytes = copy_buffer(unsafe { costing::costing_execution_cost_unit_price() });
        scrypto_decode(&bytes).unwrap()
    }

    pub fn finalization_cost_unit_limit(&mut self) -> u32 {
        unsafe { costing::costing_finalization_cost_unit_limit() }
    }

    pub fn finalization_cost_unit_price(&mut self) -> Decimal {
        let bytes = copy_buffer(unsafe { costing::costing_finalization_cost_unit_price() });
        scrypto_decode(&bytes).unwrap()
    }

    pub fn usd_price(&mut self) -> Decimal {
        let bytes = copy_buffer(unsafe { costing::costing_usd_price() });
        scrypto_decode(&bytes).unwrap()
    }

    pub fn tip_percentage(&mut self) -> u32 {
        unsafe { costing::costing_tip_percentage() }
    }

    pub fn fee_balance(&mut self) -> Decimal {
        let bytes = copy_buffer(unsafe { costing::costing_fee_balance() });
        scrypto_decode(&bytes).unwrap()
    }

    pub fn allocate_global_address(
        &mut self,
        blueprint_id: BlueprintId,
    ) -> (GlobalAddressReservation, GlobalAddress) {
        let blueprint_id = scrypto_encode(&blueprint_id).unwrap();
        let bytes = copy_buffer(unsafe {
            addr::address_allocate(blueprint_id.as_ptr(), blueprint_id.len())
        });
        scrypto_decode(&bytes).unwrap()
    }

    pub fn get_reservation_address(&mut self, node_id: &NodeId) -> GlobalAddress {
        let bytes = copy_buffer(unsafe {
            addr::address_get_reservation_address(node_id.as_ref().as_ptr(), node_id.as_ref().len())
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
            object::object_new(
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
            object::object_globalize(
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
            object::object_get_blueprint_id(node_id.as_ref().as_ptr(), node_id.as_ref().len())
        });

        scrypto_decode(&bytes).unwrap()
    }

    pub fn get_outer_object(&mut self, node_id: &NodeId) -> GlobalAddress {
        let bytes = copy_buffer(unsafe {
            object::object_get_outer_object(node_id.as_ref().as_ptr(), node_id.as_ref().len())
        });

        scrypto_decode(&bytes).unwrap()
    }

    pub fn key_value_entry_get(&mut self, handle: KeyValueEntryHandle) -> Vec<u8> {
        copy_buffer(unsafe { kv_entry::kv_entry_read(handle) })
    }

    pub fn key_value_entry_set(&mut self, handle: KeyValueEntryHandle, buffer: Vec<u8>) {
        unsafe { kv_entry::kv_entry_write(handle, buffer.as_ptr(), buffer.len()) };
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

    pub fn object_call(&mut self, receiver: &NodeId, method_name: &str, args: Vec<u8>) -> Vec<u8> {
        copy_buffer(unsafe {
            object::object_call(
                receiver.as_ref().as_ptr(),
                receiver.as_ref().len(),
                method_name.as_ptr(),
                method_name.len(),
                args.as_ptr(),
                args.len(),
            )
        })
    }

    pub fn object_call_module(
        &mut self,
        receiver: &NodeId,
        module_id: ObjectModuleId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Vec<u8> {
        copy_buffer(unsafe {
            object::object_call_module(
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
            object::object_call_direct(
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
        let blueprint_id = BlueprintId::new(&package_address, blueprint_name);
        let blueprint_id = scrypto_encode(&blueprint_id).unwrap();

        copy_buffer(unsafe {
            blueprint::blueprint_call(
                blueprint_id.as_ptr(),
                blueprint_id.len(),
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

    pub fn actor_get_object_id(&mut self, actor_ref_handle: ActorRefHandle) -> NodeId {
        let node_id = copy_buffer(unsafe { actor::actor_get_object_id(actor_ref_handle) });

        scrypto_decode(&node_id).unwrap()
    }

    pub fn actor_get_global_address(&mut self) -> GlobalAddress {
        let global_address = copy_buffer(unsafe { actor::actor_get_global_address() });

        scrypto_decode(&global_address).unwrap()
    }

    pub fn actor_get_blueprint_id(&mut self) -> BlueprintId {
        let blueprint_id = copy_buffer(unsafe { actor::actor_get_blueprint_id() });

        scrypto_decode(&blueprint_id).unwrap()
    }

    pub fn actor_emit_event(&mut self, event_name: String, event_data: Vec<u8>) {
        unsafe {
            actor::actor_emit_event(
                event_name.as_ptr(),
                event_name.len(),
                event_data.as_ptr(),
                event_data.len(),
            )
        };
    }

    pub fn sys_log(&mut self, level: Level, message: String) {
        let level = scrypto_encode(&level).unwrap();
        unsafe { system::sys_log(level.as_ptr(), level.len(), message.as_ptr(), message.len()) }
    }

    pub fn get_transaction_hash(&mut self) -> Hash {
        let actor = copy_buffer(unsafe { system::sys_get_transaction_hash() });

        scrypto_decode(&actor).unwrap()
    }

    pub fn generate_ruid(&mut self) -> [u8; 32] {
        let actor = copy_buffer(unsafe { system::sys_generate_ruid() });

        scrypto_decode(&actor).unwrap()
    }

    pub fn panic(&mut self, message: String) {
        unsafe {
            system::sys_panic(message.as_ptr(), message.len());
        };
    }
}
