use scrypto::prelude::wasm_api::*;
use scrypto::prelude::*;

const BUFFER_LENGTH: usize = 8 * 1024;

#[blueprint]
mod wasm_buffers {

    struct WasmBuffersTest {
        node_id: NodeId,
        handle: u32,
    }

    impl WasmBuffersTest {
        pub fn new() -> Global<WasmBuffersTest> {
            let schema = scrypto_encode(&KeyValueStoreGenericArgs::new::<u32, Own>(true)).unwrap();
            let bytes =
                copy_buffer(unsafe { kv_store::kv_store_new(schema.as_ptr(), schema.len()) });
            let node_id: NodeId = scrypto_decode(&bytes).unwrap();

            let key = scrypto_encode(&0u32).unwrap();

            let handle = unsafe {
                kv_store::kv_store_open_entry(
                    node_id.as_ref().as_ptr(),
                    node_id.as_ref().len(),
                    key.as_ptr(),
                    key.len(),
                    LockFlags::MUTABLE.bits(),
                )
            };

            Self { node_id, handle }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        /// Let native function "kv_entry_write" read the data from WASM memory and write it to
        /// the KV store
        pub fn read_memory(&self, offs: usize, len: usize) {
            // allocate some data but let
            let buffer = vec![0u8; BUFFER_LENGTH];

            unsafe { kv_entry::kv_entry_write(self.handle, buffer.as_ptr().add(offs), len) };
            unsafe { kv_entry::kv_entry_close(self.handle) };
        }

        /// Let native function "kv_entry_read" get the data from the KV store and then native
        /// functiion "buffer_consume" writes the data to the WASM memory
        pub fn write_memory(&self, offs: usize, len: usize) {
            let buffer = unsafe { kv_entry::kv_entry_read(self.handle) };

            // copy buffer
            let _vec = {
                //let len = buffer.len() as usize;
                let mut vec = Vec::<u8>::with_capacity(len);
                unsafe {
                    buffer::buffer_consume(buffer.id(), vec.as_mut_ptr().add(offs));
                    vec.set_len(len);
                };
                vec
            };
        }
    }
}
