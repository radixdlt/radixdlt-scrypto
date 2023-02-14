use radix_engine_interface::api::types::*;
use radix_engine_interface::api::EngineApi;
use sbor::rust::marker::PhantomData;
use scrypto::engine::scrypto_env::*;
use scrypto::prelude::*;

#[blueprint]
mod cyclic_map {
    struct CyclicMap {
        store: KeyValueStore<u32, KeyValueStore<u32, u32>>,
    }

    impl CyclicMap {
        pub fn new() -> ComponentAddress {
            let kv_store0 = KeyValueStore::new();
            let kv_store0_id = kv_store0.id.clone();
            let kv_store1 = KeyValueStore::new();
            kv_store0.insert(1u32, kv_store1);

            // Retrieve reference
            let kv_store1_id = {
                let kv_store1 = kv_store0.get(&1u32).unwrap();
                kv_store1.id.clone()
            };

            let node_id = RENodeId::KeyValueStore(kv_store1_id);
            let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(
                scrypto_encode(&0u32).unwrap(),
            ));
            let substate = KeyValueStoreEntrySubstate(Some(
                scrypto_encode(&KeyValueStore::<(), ()> {
                    id: kv_store0_id,
                    key: PhantomData,
                    value: PhantomData,
                })
                .unwrap(),
            ));

            let handle = ScryptoEnv.sys_lock_substate(node_id, offset, true).unwrap();
            ScryptoEnv
                .sys_write(handle, scrypto_encode(&substate).unwrap())
                .unwrap();

            CyclicMap { store: kv_store0 }.instantiate().globalize()
        }

        pub fn new_self_cyclic() -> ComponentAddress {
            let kv_store = KeyValueStore::new();
            let kv_store_id = kv_store.id.clone();

            let node_id = RENodeId::KeyValueStore(kv_store_id.clone());
            let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(
                scrypto_encode(&0u32).unwrap(),
            ));
            let substate = KeyValueStoreEntrySubstate(Some(
                scrypto_encode(&KeyValueStore::<(), ()> {
                    id: kv_store_id,
                    key: PhantomData,
                    value: PhantomData,
                })
                .unwrap(),
            ));

            let handle = ScryptoEnv.sys_lock_substate(node_id, offset, true).unwrap();
            ScryptoEnv
                .sys_write(handle, scrypto_encode(&substate).unwrap())
                .unwrap();

            CyclicMap { store: kv_store }.instantiate().globalize()
        }
    }
}
