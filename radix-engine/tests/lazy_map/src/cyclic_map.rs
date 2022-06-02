use scrypto::engine::api::*;
use scrypto::engine::call_engine;
use scrypto::prelude::*;
use std::marker::PhantomData;

blueprint! {
    struct CyclicMap {
        maps: LazyMap<u32, LazyMap<u32, u32>>,
    }

    impl CyclicMap {
        pub fn new() -> ComponentAddress {
            let map0 = LazyMap::new();
            let map1 = LazyMap::new();
            map0.insert(1u32, map1);

            let input = RadixEngineInput::PutLazyMapEntry(
                (Runtime::transaction_hash(), 1025),
                scrypto_encode(&0u32),
                scrypto_encode(&LazyMap::<(), ()> {
                    id: (Runtime::transaction_hash(), 1024),
                    key: PhantomData,
                    value: PhantomData,
                }),
            );
            let _: () = call_engine(input);

            CyclicMap { maps: map0 }.instantiate().globalize()
        }

        pub fn new_self_cyclic() -> ComponentAddress {
            let map0 = LazyMap::new();

            let input = RadixEngineInput::PutLazyMapEntry(
                (Runtime::transaction_hash(), 1024),
                scrypto_encode(&0u32),
                scrypto_encode(&LazyMap::<(), ()> {
                    id: (Runtime::transaction_hash(), 1024),
                    key: PhantomData,
                    value: PhantomData,
                }),
            );
            let _: () = call_engine(input);

            CyclicMap { maps: map0 }.instantiate().globalize()
        }
    }
}
