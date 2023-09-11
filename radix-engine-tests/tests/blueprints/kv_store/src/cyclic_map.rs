use scrypto::prelude::*;

#[blueprint]
mod cyclic_map {
    struct CyclicMap {
        store: KeyValueStore<u32, KeyValueStore<u32, u32>>,
    }

    impl CyclicMap {
        pub fn new() -> Global<CyclicMap> {
            let kv_store0 = KeyValueStore::new();
            let kv_store0_id = kv_store0.id.clone();
            let kv_store1 = KeyValueStore::new();
            kv_store0.insert(1u32, kv_store1);

            // Retrieve reference
            let kv_store1_id = {
                let kv_store1 = kv_store0.get(&1u32).unwrap();
                kv_store1.id.clone()
            };

            let node_id = kv_store1_id.as_node_id();
            let key = scrypto_encode(&0u32).unwrap();
            let substate = KeyValueStore::<(), ()> {
                id: kv_store0_id,
                key: PhantomData,
                value: PhantomData,
            };

            let handle = ScryptoVmV1Api::kv_store_open_entry(node_id, &key, LockFlags::MUTABLE);
            ScryptoVmV1Api::kv_entry_write(handle, scrypto_encode(&substate).unwrap());

            CyclicMap { store: kv_store0 }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn new_self_cyclic() -> Global<CyclicMap> {
            let kv_store = KeyValueStore::new();
            let kv_store_id = kv_store.id.clone();

            let node_id = kv_store_id.as_node_id();
            let key = scrypto_encode(&0u32).unwrap();
            let substate = KeyValueStore::<(), ()> {
                id: kv_store_id,
                key: PhantomData,
                value: PhantomData,
            };

            {
                let handle = ScryptoVmV1Api::kv_store_open_entry(node_id, &key, LockFlags::MUTABLE);
                ScryptoVmV1Api::kv_entry_write(handle, scrypto_encode(&substate).unwrap());
                ScryptoVmV1Api::kv_entry_close(handle);
            }

            CyclicMap { store: kv_store }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }
    }
}
