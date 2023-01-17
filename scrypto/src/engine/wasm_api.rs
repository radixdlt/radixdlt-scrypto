use radix_engine_interface::api::types::LockHandle;

pub type BufferId = u32;
pub type Buffer = u64;

extern "C" {
    //===============
    // Buffer API
    //===============

    /// Consumes a buffer by copying the contents into the specified destination.
    pub fn consume_buffer(buffer_id: BufferId, destination: *mut u8);

    //===============
    // Invocation API
    //===============

    /// Invokes a method on a scrypto component.
    pub fn invoke_method(
        receiver: *const u8,
        receive_len: usize,
        ident: *const u8,
        ident_len: usize,
        args: *const u8,
        args_len: usize,
    ) -> Buffer;

    /// Invokes any function, either scrypto or native.
    pub fn invoke(invocation: *const u8, invocation_len: usize) -> Buffer;

    //===============
    // Node API
    //===============

    /// Creates a node with the given initial data.
    pub fn create_node(init_data: *const u8, init_data_len: usize) -> Buffer;

    /// Retrieves IDs of visible nodes.
    pub fn get_visible_node_ids() -> Buffer;

    /// Destroys a node.
    pub fn drop_node(node_id: *const u8, node_id_len: usize);

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
    ) -> LockHandle;

    // Reads a substate
    pub fn read_substate(handle: LockHandle) -> Buffer;

    // Writes into a substate
    pub fn write_substate(handle: LockHandle, data: *const u8, data_len: usize);

    // Releases a lock
    pub fn unlock(handle: LockHandle);

    //===============
    // Actor API
    //===============

    // Returns the current actor.
    pub fn get_actor() -> Buffer;

}
