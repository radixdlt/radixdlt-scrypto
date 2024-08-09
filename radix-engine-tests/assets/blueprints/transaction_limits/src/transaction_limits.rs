use scrypto::prelude::*;

#[derive(Sbor, ScryptoEvent)]
struct TestEvent {
    message: String,
}

#[blueprint]
#[events(TestEvent)]
mod transaction_limits {
    struct TransactionLimitTest {
        kv_store: KeyValueStore<u32, u32>,
    }

    impl TransactionLimitTest {
        pub fn new() -> Global<TransactionLimitTest> {
            TransactionLimitTest {
                kv_store: KeyValueStore::new(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn read_non_existent_entries_from_kv_store(&self, n: u32) {
            for i in 0..n {
                self.kv_store.get(&i);
            }
        }

        pub fn write_entries_to_kv_store(&self, n: u32) {
            for i in 0..n {
                self.kv_store.insert(i, i);
            }
        }

        pub fn write_entries_to_heap_kv_store(n: u32) {
            let kv_store = KeyValueStore::new();
            for i in 0..n {
                kv_store.insert(i, i);
            }
            TransactionLimitTest { kv_store }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize();
        }

        pub fn recursive_with_memory(n: u32, m: usize) {
            if n > 1 {
                let _v: Vec<u8> = Vec::with_capacity(m);
                Blueprint::<TransactionLimitTest>::recursive_with_memory(n - 1, m);
            }
        }

        pub fn emit_event_of_size(n: usize) {
            let name = "TestEvent";
            let buf = scrypto_encode(&TestEvent {
                message: "a".repeat(n),
            })
            .unwrap();
            unsafe {
                wasm_api::actor::actor_emit_event(
                    name.as_ptr(),
                    name.len(),
                    buf.as_ptr(),
                    buf.len(),
                    0,
                )
            }
        }

        pub fn emit_log_of_size(n: usize) {
            let level = scrypto_encode(&Level::Debug).unwrap();
            let buf = "a".repeat(n);
            unsafe {
                wasm_api::system::sys_log(level.as_ptr(), level.len(), buf.as_ptr(), buf.len())
            }
        }

        pub fn panic_of_size(n: usize) {
            let buf = "a".repeat(n);
            unsafe { wasm_api::system::sys_panic(buf.as_ptr(), buf.len()) }
        }
    }
}

#[blueprint]
mod transaction_limits_substate {
    struct TransactionLimitSubstateTest {
        kv_store: KeyValueStore<u32, Vec<u8>>,
    }

    impl TransactionLimitSubstateTest {
        pub fn write_large_values(
            raw_array_sizes: Vec<usize>,
        ) -> Global<TransactionLimitSubstateTest> {
            // Create a KVStore
            let kv_store = KeyValueStore::<u32, Vec<u8>>::new();
            let mut key_value = 0u32;
            for raw_array_size in raw_array_sizes {
                // SBOR encoding of Vec<u8>
                let mut buf = Vec::new();
                let mut encoder = VecEncoder::<ScryptoCustomValueKind>::new(&mut buf, 100);
                encoder
                    .write_payload_prefix(SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)
                    .unwrap();
                encoder.write_value_kind(ValueKind::Array).unwrap();
                encoder.write_value_kind(ValueKind::U8).unwrap();
                encoder.write_size(raw_array_size).unwrap();
                buf.reserve(raw_array_size);
                let new_len = buf.len() + raw_array_size;
                unsafe { buf.set_len(new_len) };

                // Insert into store
                let key_payload = scrypto_encode(&key_value).unwrap();
                let handle = ScryptoVmV1Api::kv_store_open_entry(
                    kv_store.id.as_node_id(),
                    &key_payload,
                    LockFlags::MUTABLE,
                );
                unsafe { wasm_api::kv_entry::kv_entry_write(handle, buf.as_ptr(), buf.len()) };
                ScryptoVmV1Api::kv_entry_close(handle);

                key_value += 1;
            }

            // Put the kv store into a component
            TransactionLimitSubstateTest { kv_store }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn read_values(&self, limit: u32) {
            for key in 0..limit {
                let key_payload = scrypto_encode(&key).unwrap();
                let handle = ScryptoVmV1Api::kv_store_open_entry(
                    self.kv_store.id.as_node_id(),
                    &key_payload,
                    LockFlags::read_only(),
                );
                let _raw_bytes = ScryptoVmV1Api::kv_entry_read(handle);
                ScryptoVmV1Api::kv_entry_close(handle);
            }
        }
    }
}

#[blueprint]
mod invoke_limits {
    struct InvokeLimitsTest {}

    impl InvokeLimitsTest {
        pub fn call(raw_array_size: usize) {
            // SBOR encoding of (Vec<u8>)
            let mut buf = Vec::new();
            let mut encoder = VecEncoder::<ScryptoCustomValueKind>::new(&mut buf, 100);
            encoder
                .write_payload_prefix(SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)
                .unwrap();
            encoder.write_value_kind(ValueKind::Tuple).unwrap();
            encoder.write_size(1).unwrap();
            encoder.write_value_kind(ValueKind::Array).unwrap();
            encoder.write_value_kind(ValueKind::U8).unwrap();
            encoder.write_size(raw_array_size).unwrap();
            buf.reserve(raw_array_size);
            let new_len = buf.len() + raw_array_size;
            unsafe { buf.set_len(new_len) };

            ScryptoVmV1Api::blueprint_call(
                Runtime::package_address(),
                "InvokeLimitsTest",
                "callee",
                buf,
            );
        }

        pub fn callee(_: Vec<u8>) {}
    }
}

#[blueprint]
mod buffer_limit {
    struct BufferLimit {
        state: Vec<u8>,
    }

    impl BufferLimit {
        pub fn new() -> Global<BufferLimit> {
            BufferLimit {
                state: [0u8; 200 * 1024].to_vec(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn allocate_buffers(&self, n: u32) {
            let handle =
                ScryptoVmV1Api::actor_open_field(ACTOR_STATE_SELF, 0u8, LockFlags::empty());
            for _ in 0..n {
                unsafe {
                    scrypto::engine::wasm_api::field_entry::field_entry_read(handle);
                }
            }
        }
    }
}
