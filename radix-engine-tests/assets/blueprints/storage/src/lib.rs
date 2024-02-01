use scrypto::prelude::*;

#[blueprint]
mod large_key {
    struct LargeKey {
        kv_store: KeyValueStore<Vec<u8>, ()>,
    }

    impl LargeKey {
        pub fn create_kv_store_with_many_large_keys(n: u32) {
            let kv_store = KeyValueStore::new();
            let mut key_payload = scrypto_encode(&[0u8; 1000]).unwrap();
            let value_payload = scrypto_encode(&()).unwrap();

            for i in 0..n {
                let n = key_payload.len();
                key_payload[n - 4..n].copy_from_slice(&i.to_le_bytes());
                let handle = ScryptoVmV1Api::kv_store_open_entry(
                    kv_store.id.as_node_id(),
                    &key_payload,
                    LockFlags::MUTABLE,
                );
                unsafe {
                    wasm_api::kv_entry::kv_entry_write(
                        handle,
                        value_payload.as_ptr(),
                        value_payload.len(),
                    )
                };
                ScryptoVmV1Api::kv_entry_close(handle);
            }

            LargeKey { kv_store }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize();
        }
    }
}
