use scrypto::engine::api::*;
use scrypto::engine::call_engine;
use scrypto::prelude::*;
use std::marker::PhantomData;

blueprint! {
    struct CyclicMap {
        maps: LazyMap<u32, LazyMap<u32, u32>>,
    }

    impl CyclicMap {
        pub fn new() -> ComponentId {
            let map0 = LazyMap::new();
            let map1 = LazyMap::new();
            map0.insert(1u32, map1);

            let input = PutLazyMapEntryInput {
                lazy_map_id: (Transaction::transaction_hash(), 1025),
                key: scrypto_encode(&0u32),
                value: scrypto_encode(&LazyMap::<(), ()> {
                    id: (Transaction::transaction_hash(), 1024),
                    key: PhantomData,
                    value: PhantomData,
                }),
            };
            let _: PutLazyMapEntryOutput = call_engine(PUT_LAZY_MAP_ENTRY, input);

            CyclicMap { maps: map0 }.instantiate().globalize()
        }

        pub fn new_self_cyclic() -> ComponentId {
            let map0 = LazyMap::new();

            let input = PutLazyMapEntryInput {
                lazy_map_id: (Transaction::transaction_hash(), 1024),
                key: scrypto_encode(&0u32),
                value: scrypto_encode(&LazyMap::<(), ()> {
                    id: (Transaction::transaction_hash(), 1024),
                    key: PhantomData,
                    value: PhantomData,
                }),
            };
            let _: PutLazyMapEntryOutput = call_engine(PUT_LAZY_MAP_ENTRY, input);

            CyclicMap { maps: map0 }.instantiate().globalize()
        }
    }
}
