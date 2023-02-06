use scrypto::prelude::*;

#[blueprint]
mod super_key_value_store {
    struct SuperKeyValueStore {
        maps: KeyValueStore<
            u32,
            KeyValueStore<u32, KeyValueStore<u32, KeyValueStore<String, String>>>,
        >,
    }

    impl SuperKeyValueStore {
        pub fn new() -> ComponentAddress {
            let map0 = KeyValueStore::new();
            let map1 = KeyValueStore::new();
            let map2 = KeyValueStore::new();
            let map3 = KeyValueStore::new();
            let map4 = KeyValueStore::new();
            map2.insert(3u32, map3);
            map0.insert(1u32, map1);
            map0.insert(2u32, map2);

            {
                let map2 = map0.get(&2u32).unwrap();
                let map3 = map2.get(&3u32).unwrap();
                map3.insert(4u32, map4);
            }

            SuperKeyValueStore { maps: map0 }.instantiate().globalize()
        }
    }
}
