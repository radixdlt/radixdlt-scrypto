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
            let key_value_store0 = KeyValueStore::new();
            let key_value_store0_id = key_value_store0.id.clone();
            let key_value_store1 = KeyValueStore::new();
            key_value_store0.insert(1u32, key_value_store1);

            // Retrieve reference
            let key_value_store1_id = {
                let key_value_store1 = key_value_store0.get(&1u32).unwrap();
                key_value_store1.id.clone()
            };

            let substate_id =
                SubstateId::KeyValueStoreEntry(key_value_store1_id, scrypto_encode(&0u32));
            let input = RadixEngineInput::SubstateWrite(
                substate_id,
                scrypto_encode(&KeyValueStore::<(), ()> {
                    id: key_value_store0_id,
                    key: PhantomData,
                    value: PhantomData,
                }),
            );
            let _: () = call_engine(input);

            CyclicMap {
                store: key_value_store0,
            }
            .instantiate()
            .globalize()
        }

        pub fn new_self_cyclic() -> ComponentAddress {
            let key_value_store = KeyValueStore::new();
            let key_value_store_id = key_value_store.id.clone();

            let substate_id =
                SubstateId::KeyValueStoreEntry(key_value_store_id.clone(), scrypto_encode(&0u32));
            let input = RadixEngineInput::SubstateWrite(
                substate_id,
                scrypto_encode(&KeyValueStore::<(), ()> {
                    id: key_value_store_id,
                    key: PhantomData,
                    value: PhantomData,
                }),
            );
            let _: () = call_engine(input);

            CyclicMap {
                store: key_value_store,
            }
            .instantiate()
            .globalize()
        }
    }
}
