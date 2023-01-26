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

extern "C" {
    //===============
    // Buffer API
    //===============

    /// Consumes a buffer by copying the contents into the specified destination.
    pub fn consume_buffer(buffer_id: BufferId, destination_ptr: *mut u8);

    //===============
    // Invocation API
    //===============

    /// Invokes a method on a component.
    pub fn call_method(
        receiver_ptr: *const u8,
        receive_len: usize,
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

    /// Invokes a native function or method.
    pub fn call_native(
        native_fn_identifier_ptr: *const u8,
        native_fn_identifier_len: usize,
        invocation_ptr: *const u8,
        invocation_len: usize,
    ) -> Buffer;

    //===============
    // Node API
    //===============

    /// Creates a node with the given initial data.
    pub fn create_node(node_ptr: *const u8, node_len: usize) -> Buffer;

    /// Retrieves IDs of visible nodes.
    pub fn get_visible_nodes() -> Buffer;

    /// Destroys a node.
    pub fn drop_node(node_id_ptr: *const u8, node_id_len: usize);

    //===============
    // Substate API
    //===============

    // Locks a substate
    pub fn lock_substate(
        node_id: *const u8,
        node_id_len: usize,
        offset: *const u8,
        offset_len: usize,
        mutable: bool,
    ) -> u32;

    // Reads a substate
    pub fn read_substate(handle: u32) -> Buffer;

    // Writes into a substate
    pub fn write_substate(handle: u32, data_ptr: *const u8, data_len: usize);

    // Releases a lock
    pub fn unlock_substate(handle: u32);

    //===============
    // Actor API
    //===============

    // Returns the current actor.
    pub fn get_actor() -> Buffer;
}
