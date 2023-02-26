// Re-export
pub use radix_engine_interface::api::types::{Buffer, BufferId, Slice};

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

    pub fn new_component(
        blueprint_ident_ptr: *const u8,
        blueprint_ident: usize,
        app_states_ptr: *const u8,
        app_states_len: usize,
    ) -> Buffer;

    pub fn new_key_value_store() -> Buffer;

    pub fn globalize_component(
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

    pub fn get_component_type_info(component_id_ptr: *const u8, component_id_len: usize) -> Buffer;

    /// Invokes a method on a component.
    pub fn call_method(
        receiver_ptr: *const u8,
        receive_len: usize,
        node_module_id: u32,
        ident_ptr: *const u8,
        ident_len: usize,
        args_ptr: *const u8,
        args_len: usize,
    ) -> Buffer;

    //===============
    // Package API
    //===============

    pub fn new_package(
        code_ptr: *const u8,
        code_len: usize,
        abi_ptr: *const u8,
        abi_len: usize,
        access_rules_chain_ptr: *const u8,
        access_rules_chain: usize,
        royalty_config_ptr: *const u8,
        royalty_config: usize,
        metadata_ptr: *const u8,
        metadata_len: usize,
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

    //===============
    // Node API
    //===============

    /// Creates a node with the given initial data.
    pub fn create_node(node_ptr: *const u8, node_len: usize) -> Buffer;

    /// Destroys a node.
    pub fn drop_node(node_id_ptr: *const u8, node_id_len: usize);

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
    // Actor API
    //===============

    // Returns the current actor.
    pub fn get_actor() -> Buffer;
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn consume_buffer(_buffer_id: BufferId, _destination_ptr: *mut u8) {
    todo!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn new_component(
    _blueprint_ident_ptr: *const u8,
    _blueprint_ident: usize,
    _app_states_ptr: *const u8,
    _app_states: usize,
) -> Buffer {
    todo!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn new_key_value_store() -> Buffer {
    todo!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn globalize_component(
    _node_id_ptr: *const u8,
    _node_id_len: usize,
    _modules_ptr: *const u8,
    _modules_len: usize,
) -> Buffer {
    todo!()
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
    todo!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn get_component_type_info(
    _component_id_ptr: *const u8,
    _component_id_len: usize,
) -> Buffer {
    todo!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn call_method(
    _receiver_ptr: *const u8,
    _receive_len: usize,
    _node_module_id: u32,
    _ident_ptr: *const u8,
    _ident_len: usize,
    _args_ptr: *const u8,
    _args_len: usize,
) -> Buffer {
    todo!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn new_package(
    _code_ptr: *const u8,
    _code_len: usize,
    _abi_ptr: *const u8,
    _abi_len: usize,
    _access_rules_chain_ptr: *const u8,
    _access_rules_chain: usize,
    _royalty_config_ptr: *const u8,
    _royalty_config: usize,
    _metadata_ptr: *const u8,
    _metadata_len: usize,
) -> Buffer {
    todo!()
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
    todo!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn drop_node(_node_id_ptr: *const u8, _node_id_len: usize) {
    todo!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn lock_substate(
    _node_id: *const u8,
    _node_id_len: usize,
    _offset: *const u8,
    _offset_len: usize,
    _flags: u32,
) -> u32 {
    todo!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn read_substate(_handle: u32) -> Buffer {
    todo!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn write_substate(_handle: u32, _data_ptr: *const u8, _data_len: usize) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn drop_lock(_handle: u32) {
    todo!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn get_actor() -> Buffer {
    todo!()
}
