pub type BufferId = u32;
pub type Buffer = u64;

#[macro_export]
macro_rules! buffer_id {
    ($buf: expr) => {
        ($buf >> 32) as u32
    };
}

#[macro_export]
macro_rules! buffer_len {
    ($buf: expr) => {
        ($buf & 0xffffffff) as usize
    };
}

pub fn copy_buffer(buffer: Buffer) -> Vec<u8> {
    let mut vec = Vec::<u8>::with_capacity(buffer_len!(buffer));
    unsafe {
        consume_buffer(buffer_id!(buffer), vec.as_mut_ptr());
        vec.set_len(buffer_len!(buffer));
    };
    vec
}

pub type Slice = u64;

pub fn forget_vec(vec: Vec<u8>) -> Slice {
    let ptr = vec.as_ptr() as u64;
    let len = vec.len() as u64;
    assert!(ptr < 0xffffffff && ptr < 0xffffffff);

    // Note that hhe memory used by the Vec is forever leaked.
    // However, it's not an issue since the wasm instance will be destroyed after engine
    // consuming the data.
    sbor::rust::mem::forget(vec);

    (ptr << 32) | len
}

extern "C" {
    //===============
    // Buffer API
    //===============

    /// Consumes a buffer by copying the contents into the specified destination.
    pub fn consume_buffer(buffer_id: BufferId, destination: *mut u8);

    //===============
    // Invocation API
    //===============

    /// Invokes a method on a component.
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
    pub fn create_node(node: *const u8, node_len: usize) -> Buffer;

    /// Retrieves IDs of visible nodes.
    pub fn get_visible_nodes() -> Buffer;

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
    ) -> u32;

    // Reads a substate
    pub fn read_substate(handle: u32) -> Buffer;

    // Writes into a substate
    pub fn write_substate(handle: u32, data: *const u8, data_len: usize);

    // Releases a lock
    pub fn unlock_substate(handle: u32);

    //===============
    // Actor API
    //===============

    // Returns the current actor.
    pub fn get_actor() -> Buffer;

}
