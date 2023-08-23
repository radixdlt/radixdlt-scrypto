// Re-export
pub use radix_engine_interface::types::{Buffer, BufferId, Slice};

use sbor::rust::vec::Vec;

pub fn copy_buffer(buffer: Buffer) -> Vec<u8> {
    let len = buffer.len() as usize;
    let mut vec = Vec::<u8>::with_capacity(len);
    unsafe {
        buffer::buffer_consume(buffer.id(), vec.as_mut_ptr());
        vec.set_len(len);
    };
    vec
}

pub fn forget_vec(vec: Vec<u8>) -> Slice {
    let ptr = vec.as_ptr() as usize;
    let len = vec.len();
    assert!(ptr <= 0xffffffff && len <= 0xffffffff);

    // Note that the memory used by the Vec is forever leaked.
    // However, it's not an issue since the wasm instance will be destroyed after engine
    // consuming the data.
    sbor::rust::mem::forget(vec);

    Slice::new(ptr as u32, len as u32)
}

/// Api make blueprint function calls
pub mod blueprint {
    pub use radix_engine_interface::types::{Buffer, BufferId, Slice};

    extern "C" {
        /// Invokes a blueprint function
        pub fn blueprint_call(
            blueprint_id_ptr: *const u8,
            blueprint_id_len: usize,
            ident_ptr: *const u8,
            ident_len: usize,
            args_ptr: *const u8,
            args_len: usize,
        ) -> Buffer;
    }
}

/// API to allocate/reserve global address
pub mod addr {
    pub use radix_engine_interface::types::{Buffer, BufferId, Slice};

    extern "C" {
        /// Reserves a global address for a given blueprint
        pub fn address_allocate(blueprint_id_ptr: *const u8, blueprint_id_len: usize) -> Buffer;

        /// Get the address associated with an address reservation
        pub fn address_get_reservation_address(
            address_id_ptr: *const u8,
            address_id_len: usize,
        ) -> Buffer;
    }
}

/// API to manipulate or get information about visible objects
pub mod object {
    pub use radix_engine_interface::types::{Buffer, BufferId, Slice};

    extern "C" {
        /// Creates a new object of a given blueprint defined in the same
        /// package as the current actor
        pub fn object_new(
            blueprint_id_ptr: *const u8,
            blueprint_id: usize,
            obj_fields_ptr: *const u8,
            obj_fields_len: usize,
        ) -> Buffer;

        /// Globalizes an object with given modules
        pub fn object_globalize(
            obj_id_ptr: *const u8,
            obj_id_len: usize,
            modules_ptr: *const u8,
            modules_len: usize,
            address_id_ptr: *const u8,
            address_id_len: usize,
        ) -> Buffer;

        /// Get the Blueprint Identifier of a given object
        pub fn object_get_blueprint_id(obj_id_ptr: *const u8, obj_id_len: usize) -> Buffer;

        /// Get the address of the outer object of a given object
        pub fn object_get_outer_object(obj_id_ptr: *const u8, obj_id_len: usize) -> Buffer;

        /// Invokes a method on a visible object
        pub fn object_call(
            obj_id_ptr: *const u8,
            obj_id_len: usize,
            ident_ptr: *const u8,
            ident_len: usize,
            args_ptr: *const u8,
            args_len: usize,
        ) -> Buffer;

        /// Invokes a direct method on a visible object
        pub fn object_call_direct(
            obj_id_ptr: *const u8,
            obj_id_len: usize,
            ident_ptr: *const u8,
            ident_len: usize,
            args_ptr: *const u8,
            args_len: usize,
        ) -> Buffer;

        /// Invokes a module method on a visible object
        pub fn object_call_module(
            obj_id_ptr: *const u8,
            obj_id_len: usize,
            module_id: u32,
            ident_ptr: *const u8,
            ident_len: usize,
            args_ptr: *const u8,
            args_len: usize,
        ) -> Buffer;
    }
}

/// API to manipulate or get information about the current actor
pub mod actor {
    use radix_engine_interface::api::field_api::FieldHandle;
    use radix_engine_interface::api::{ActorRefHandle, ActorStateHandle};
    pub use radix_engine_interface::types::{Buffer, BufferId, Slice};

    extern "C" {
        /// Get the blueprint id of the current actor
        pub fn actor_get_blueprint_id() -> Buffer;

        /// Get the object id of a reference of the current actor
        pub fn actor_get_object_id(actor_ref_handle: ActorRefHandle) -> Buffer;

        /// Open a field of the current actor
        pub fn actor_open_field(
            actor_state_handle: ActorStateHandle,
            field_index: u32,
            flags: u32,
        ) -> FieldHandle;

        /// Emit an event
        pub fn actor_emit_event(
            event_name_ptr: *const u8,
            event_name_len: usize,
            event_data_ptr: *const u8,
            event_data_len: usize,
            event_flags: u32,
        );
    }
}

pub mod kv_store {
    pub use radix_engine_interface::types::{Buffer, BufferId, Slice};

    extern "C" {
        /// Creates a new key value store
        pub fn kv_store_new(schema_ptr: *const u8, schema_len: usize) -> Buffer;

        /// Opens an entry for a given key in a key value store
        pub fn kv_store_open_entry(
            key_value_store_id_ptr: *const u8,
            key_value_store_id_len: usize,
            key_ptr: *const u8,
            key_len: usize,
            flags: u32,
        ) -> u32;

        /// Removes a value from a key value store
        pub fn kv_store_remove_entry(
            key_value_store_id_ptr: *const u8,
            key_value_store_id_len: usize,
            key: *const u8,
            key_len: usize,
        ) -> Buffer;
    }
}

/// API to manipulate or get information about an open Key Value Entry
pub mod kv_entry {
    pub use radix_engine_interface::types::{Buffer, BufferId, Slice};

    extern "C" {
        /// Reads the value in a Key Value entry
        pub fn kv_entry_read(kv_entry_handle: u32) -> Buffer;

        /// Writes a value to Key Value entry
        pub fn kv_entry_write(kv_entry_handle: u32, buffer_ptr: *const u8, buffer_len: usize);

        /// Removes the value in an underlying Key Value entry
        pub fn kv_entry_remove(kv_entry_handle: u32) -> Buffer;

        /// Close a Key Value entry
        pub fn kv_entry_close(kv_entry_handle: u32);
    }
}

/// API to manipulate or get information about an open Field Entry
pub mod field_entry {
    pub use radix_engine_interface::types::{Buffer, BufferId, Slice};

    extern "C" {
        /// Reads the value in a field
        pub fn field_entry_read(handle: u32) -> Buffer;

        /// Writes a value to a field
        pub fn field_entry_write(handle: u32, data_ptr: *const u8, data_len: usize);

        /// Close a field entry
        pub fn field_entry_close(handle: u32);
    }
}

pub mod costing {
    pub use radix_engine_interface::types::{Buffer, BufferId, Slice};

    extern "C" {
        pub fn costing_get_execution_cost_unit_limit() -> u32;

        pub fn costing_get_execution_cost_unit_price() -> Buffer;

        pub fn costing_get_finalization_cost_unit_limit() -> u32;

        pub fn costing_get_finalization_cost_unit_price() -> Buffer;

        pub fn costing_get_usd_price() -> Buffer;

        pub fn costing_get_tip_percentage() -> u32;

        pub fn costing_get_fee_balance() -> Buffer;
    }
}

/// Various environment-based API calls
pub mod system {
    pub use radix_engine_interface::types::{Buffer, BufferId, Slice};

    extern "C" {
        /// Logs a string message
        pub fn sys_log(
            level_ptr: *const u8,
            level_len: usize,
            message_ptr: *const u8,
            message_len: usize,
        );

        /// Encode an address to bech32 encoding
        pub fn sys_bech32_encode_address(address_ptr: *const u8, address_len: usize) -> Buffer;

        /// Retrieves the current transaction hash
        pub fn sys_get_transaction_hash() -> Buffer;

        /// Generates a unique id
        pub fn sys_generate_ruid() -> Buffer;

        /// Panics and halts transaction execution
        pub fn sys_panic(message_ptr: *const u8, message_len: usize);
    }
}

pub mod buffer {
    pub use radix_engine_interface::types::{Buffer, BufferId, Slice};

    extern "C" {
        /// Consumes a buffer by copying the contents into the specified destination.
        pub fn buffer_consume(buffer_id: BufferId, destination_ptr: *mut u8);
    }
}
