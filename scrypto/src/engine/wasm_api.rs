// Re-export
pub use radix_engine_interface::types::{Buffer, BufferId, Slice};

use sbor::rust::vec::Vec;

pub fn copy_buffer(buffer: Buffer) -> Vec<u8> {
    let len = buffer.len() as usize;
    let mut vec = Vec::<u8>::with_capacity(len);
    unsafe {
        consume_buffer(buffer.id(), vec.as_mut_ptr());
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

#[cfg(target_arch = "wasm32")]
extern "C" {
    //===============
    // Buffer API
    //===============

    /// Consumes a buffer by copying the contents into the specified destination.
    pub fn consume_buffer(buffer_id: BufferId, destination_ptr: *mut u8);

    //===============
    // Object API
    //===============

    pub fn new_object(
        blueprint_ident_ptr: *const u8,
        blueprint_ident: usize,
        object_states_ptr: *const u8,
        object_states_len: usize,
    ) -> Buffer;

    pub fn new_key_value_store(schema_ptr: *const u8, schema_len: usize) -> Buffer;

    pub fn globalize_object(
        component_id_ptr: *const u8,
        component_id_len: usize,
        modules_ptr: *const u8,
        modules_len: usize,
    ) -> Buffer;

    pub fn globalize_with_address(
        _node_id_ptr: *const u8,
        _node_id_len: usize,
        _modules_ptr: *const u8,
        _modules_len: usize,
        _address_ptr: *const u8,
        _address_len: usize,
    ) -> Buffer;

    pub fn get_object_info(component_id_ptr: *const u8, component_id_len: usize) -> Buffer;

    pub fn get_key_value_store_info(
        key_value_store_id_ptr: *const u8,
        key_value_store_id_len: usize,
    ) -> Buffer;

    /// Invokes a method on a component.
    pub fn call_method(
        receiver_ptr: *const u8,
        receive_len: usize,
        module_id: u32,
        ident_ptr: *const u8,
        ident_len: usize,
        args_ptr: *const u8,
        args_len: usize,
    ) -> Buffer;

    /// Invokes a function on a blueprint.
    pub fn call_function(
        package_address_ptr: *const u8,
        package_address_len: usize,
        blueprint_ident_ptr: *const u8,
        blueprint_ident_len: usize,
        function_ident_ptr: *const u8,
        function_ident_len: usize,
        args_ptr: *const u8,
        args_len: usize,
    ) -> Buffer;

    /// Destroys a node.
    pub fn drop_object(node_id_ptr: *const u8, node_id_len: usize);

    //===============
    // Substate API
    //===============

    // Locks a substate
    pub fn lock_substate(
        node_id_ptr: *const u8,
        node_id_len: usize,
        offset_ptr: *const u8,
        offset_len: usize,
        flags: u32,
    ) -> u32;

    // Reads a substate
    pub fn read_substate(handle: u32) -> Buffer;

    // Writes into a substate
    pub fn write_substate(handle: u32, data_ptr: *const u8, data_len: usize);

    // Releases a lock
    pub fn drop_lock(handle: u32);

    //===============
    // System API
    //===============

    pub fn get_global_address() -> Buffer;

    pub fn get_blueprint() -> Buffer;

    pub fn get_auth_zone() -> Buffer;

    pub fn assert_access_rule(rule_ptr: *const u8, rule_len: usize);

    pub fn emit_event(
        event_name_ptr: *const u8,
        event_name_len: usize,
        event_data_ptr: *const u8,
        event_data_len: usize,
    );

    pub fn log_message(
        level_ptr: *const u8,
        level_len: usize,
        message_ptr: *const u8,
        message_len: usize,
    );

    pub fn get_transaction_hash() -> Buffer;

    pub fn generate_uuid() -> Buffer;
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn consume_buffer(_buffer_id: BufferId, _destination_ptr: *mut u8) {
    unreachable!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn new_object(
    _blueprint_ident_ptr: *const u8,
    _blueprint_ident: usize,
    _object_states_ptr: *const u8,
    _object_states: usize,
) -> Buffer {
    unreachable!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn new_key_value_store(_schema_ptr: *const u8, _schema_len: usize) -> Buffer {
    unreachable!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn globalize_object(
    _node_id_ptr: *const u8,
    _node_id_len: usize,
    _modules_ptr: *const u8,
    _modules_len: usize,
) -> Buffer {
    unreachable!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn globalize_with_address(
    _node_id_ptr: *const u8,
    _node_id_len: usize,
    _modules_ptr: *const u8,
    _modules_len: usize,
    _address_ptr: *const u8,
    _address_len: usize,
) -> Buffer {
    unreachable!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn get_object_info(_component_id_ptr: *const u8, _component_id_len: usize) -> Buffer {
    unreachable!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn get_key_value_store_info(
    _key_value_store_id_ptr: *const u8,
    _key_value_store_id_len: usize,
) -> Buffer {
    unreachable!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn call_method(
    _receiver_ptr: *const u8,
    _receive_len: usize,
    _module_id: u32,
    _ident_ptr: *const u8,
    _ident_len: usize,
    _args_ptr: *const u8,
    _args_len: usize,
) -> Buffer {
    unreachable!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn call_function(
    _package_address_ptr: *const u8,
    _package_address_len: usize,
    _blueprint_ident_ptr: *const u8,
    _blueprint_ident_len: usize,
    _function_ident_ptr: *const u8,
    _function_ident_len: usize,
    _args_ptr: *const u8,
    _args_len: usize,
) -> Buffer {
    unreachable!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn drop_object(_node_id_ptr: *const u8, _node_id_len: usize) {
    unreachable!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn lock_substate(
    _node_id: *const u8,
    _node_id_len: usize,
    _offset: *const u8,
    _offset_len: usize,
    _flags: u32,
) -> u32 {
    unreachable!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn read_substate(_handle: u32) -> Buffer {
    unreachable!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn write_substate(_handle: u32, _data_ptr: *const u8, _data_len: usize) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn drop_lock(_handle: u32) {
    unreachable!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn get_global_address() -> Buffer {
    unreachable!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn get_blueprint() -> Buffer {
    unreachable!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn get_auth_zone() -> Buffer {
    unreachable!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn assert_access_rule(_rule_ptr: *const u8, _rule_len: usize) {
    unreachable!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn emit_event(
    _event_name_ptr: *const u8,
    _event_name_len: usize,
    _event_data_ptr: *const u8,
    _event_data_len: usize,
) {
    unreachable!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn log_message(
    _level_ptr: *const u8,
    _level_len: usize,
    _message_ptr: *const u8,
    _message_len: usize,
) {
    unreachable!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn get_transaction_hash() -> Buffer {
    unreachable!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn generate_uuid() -> Buffer {
    unreachable!()
}
