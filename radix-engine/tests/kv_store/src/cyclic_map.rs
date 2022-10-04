use scrypto::engine::api::*;
use scrypto::engine::call_engine;
use scrypto::engine::types::*;
use scrypto::prelude::*;
use std::marker::PhantomData;

blueprint! {
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

            let substate_id = SubstateId(RENodeId::KeyValueStore(
                kv_store1_id),
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(scrypto_encode(&0u32))),
            );
            let input = RadixEngineInput::SubstateWrite(
                substate_id,
                scrypto_encode(&KeyValueStore::<(), ()> {
                    id: kv_store0_id,
                    key: PhantomData,
                    value: PhantomData,
                }),
            );
            let _: () = call_engine(input);

            CyclicMap { store: kv_store0 }.instantiate().globalize()
        }

        pub fn new_self_cyclic() -> ComponentAddress {
            let kv_store = KeyValueStore::new();
            let kv_store_id = kv_store.id.clone();

            let substate_id = SubstateId(RENodeId::KeyValueStore(
                kv_store_id.clone()),
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(scrypto_encode(&0u32))),
            );
            let input = RadixEngineInput::SubstateWrite(
                substate_id,
                scrypto_encode(&KeyValueStore::<(), ()> {
                    id: kv_store_id,
                    key: PhantomData,
                    value: PhantomData,
                }),
            );
            let _: () = call_engine(input);

            CyclicMap { store: kv_store }.instantiate().globalize()
        }
    }
}
