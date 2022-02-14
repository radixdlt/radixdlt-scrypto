use scrypto::prelude::*;

blueprint! {
    struct SuperLazyMap {
        maps: LazyMap<u32, LazyMap<u32, LazyMap<u32, LazyMap<String, String>>>>
    }

    impl SuperLazyMap {
        pub fn new() -> ComponentRef {
            let map0 = LazyMap::new();
            let map1 = LazyMap::new();
            let map2 = LazyMap::new();
            let map3 = LazyMap::new();
            let map4 = LazyMap::new();
            map2.insert(3u32, map3);
            map0.insert(1u32, map1);
            map0.insert(2u32, map2);

            let map2 = map0.get(&2u32).unwrap();
            let map3 = map2.get(&3u32).unwrap();
            map3.insert(4u32, map4);

            SuperLazyMap {
                maps: map0
            }.instantiate()
        }
    }
}
