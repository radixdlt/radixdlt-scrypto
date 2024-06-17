use scrypto::prelude::wasm_api::*;
use scrypto::prelude::*;

#[blueprint]
mod system_wasm_buffers {

    struct WasmBuffersTest {
        kv_store: KeyValueStore<u32, Vec<u8>>,
        key: Vec<u8>,
    }

    impl WasmBuffersTest {
        pub fn new() -> Global<WasmBuffersTest> {
            let kv_store = KeyValueStore::<u32, Vec<u8>>::new();
            let key = scrypto_encode(&1u32).unwrap();

            Self { kv_store, key }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        fn get_kv_store_handle(&self) -> u32 {
            let node_id = self.kv_store.id.as_node_id();
            let handle = unsafe {
                kv_store::kv_store_open_entry(
                    node_id.as_bytes().as_ptr(),
                    node_id.as_bytes().len(),
                    self.key.as_ptr(),
                    self.key.len(),
                    LockFlags::MUTABLE.bits(),
                )
            };
            handle
        }

        /// Let native function "kv_entry_write" read the data from WASM memory and write it to
        /// the KV store
        /// Arguments:
        /// - buffer_size - WASM buffer to allocate
        /// - read_memory_offs - buffer offset to start reading data from
        /// - read_memory_len - number of bytes to read from memory
        /// WASM memory grows in 64KB chunks.
        /// If attempting to access outside WASM memory, make sure that
        /// read_memory_offset + read_memory_len > buffer_size + 64KB
        pub fn read_memory(
            &self,
            buffer_size: usize,
            read_memory_offs: isize,
            read_memory_len: usize,
        ) {
            // SBOR encoding of Vec<u8>
            let mut buffer = Vec::new();
            let mut encoder = VecEncoder::<ScryptoCustomValueKind>::new(&mut buffer, 100);
            encoder
                .write_payload_prefix(SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)
                .unwrap();
            encoder.write_value_kind(ValueKind::Array).unwrap();
            encoder.write_value_kind(ValueKind::U8).unwrap();
            encoder.write_size(buffer_size).unwrap();
            buffer.reserve(buffer_size);
            let new_size = buffer.len() + buffer_size;
            unsafe { buffer.set_len(new_size) };

            let handle = self.get_kv_store_handle();

            unsafe {
                let mut buffer_ptr = buffer.as_ptr();
                buffer_ptr = if read_memory_offs < 0 {
                    buffer_ptr.sub((-read_memory_offs) as usize)
                } else {
                    buffer_ptr.add(read_memory_offs as usize)
                };

                kv_entry::kv_entry_write(handle, buffer_ptr, read_memory_len)
            };
            unsafe { kv_entry::kv_entry_close(handle) };
        }

        /// Let native function "kv_entry_read" get the data from the KV store and then native
        /// function "buffer_consume" writes the data to the WASM memory
        /// Arguments:
        /// - buffer_size - WASM buffer to allocate
        /// - write_memory_offs - buffer offset to start reading data from
        /// WASM memory grows in 64KB chunks.
        /// If attempting to access outside WASM memory, make sure that
        /// write_memory_offs > buffer_size + 64KB
        pub fn write_memory(&self, buffer_size: usize, write_memory_offs: isize) {
            let handle = self.get_kv_store_handle();
            let buffer = unsafe { kv_entry::kv_entry_read(handle) };

            // copy buffer
            let mut vec = Vec::<u8>::with_capacity(buffer_size);
            unsafe {
                let mut vec_ptr = vec.as_mut_ptr();
                vec_ptr = if write_memory_offs < 0 {
                    vec_ptr.sub((-write_memory_offs) as usize)
                } else {
                    vec_ptr.add(write_memory_offs as usize)
                };

                buffer::buffer_consume(buffer.id(), vec_ptr);
                vec.set_len(buffer_size);
            }
            unsafe { kv_entry::kv_entry_close(handle) };
        }

        pub fn write_memory_specific_buffer_id(&self, buffer_id: u32) {
            let handle = self.get_kv_store_handle();
            let buffer = unsafe { kv_entry::kv_entry_read(handle) };
            let buffer_size = buffer.len() as usize;

            // copy buffer
            let mut vec = Vec::<u8>::with_capacity(buffer_size);
            unsafe {
                let vec_ptr = vec.as_mut_ptr();

                // use specified buffer id
                buffer::buffer_consume(buffer_id, vec_ptr);
                vec.set_len(buffer_size);
            }
            unsafe { kv_entry::kv_entry_close(handle) };
        }

        pub fn write_memory_specific_buffer_ptr(&self, buffer_ptr: u32) {
            let handle = self.get_kv_store_handle();
            let buffer = unsafe { kv_entry::kv_entry_read(handle) };

            // copy buffer
            unsafe {
                let vec_ptr: *mut u8 = buffer_ptr as *mut u8;

                // use specified buffer ptr
                buffer::buffer_consume(buffer.id(), vec_ptr);
            }
            unsafe { kv_entry::kv_entry_close(handle) };
        }
    }
}
