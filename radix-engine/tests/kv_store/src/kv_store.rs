use scrypto::prelude::*;

blueprint! {
    struct LazyMapTest {
        map: KeyValueStore<String, String>,
        vector: Vec<KeyValueStore<String, String>>,
        lazy_maps: KeyValueStore<String, KeyValueStore<String, String>>,
    }

    impl LazyMapTest {
        pub fn dangling_lazy_map() -> Option<String> {
            let map = KeyValueStore::new();
            map.insert("hello".to_owned(), "world".to_owned());
            map.get(&"hello".to_owned())
        }

        pub fn new_lazy_map_into_vector() -> ComponentAddress {
            let map = KeyValueStore::new();
            map.get(&"hello".to_owned());
            let mut vector = Vec::new();
            vector.push(KeyValueStore::new());
            let lazy_maps = KeyValueStore::new();
            LazyMapTest {
                map,
                vector,
                lazy_maps,
            }
            .instantiate()
            .globalize()
        }

        pub fn new_lazy_map_into_lazy_map() -> ComponentAddress {
            let map = KeyValueStore::new();
            let vector = Vec::new();
            let lazy_maps = KeyValueStore::new();
            lazy_maps.insert("hello".to_owned(), KeyValueStore::new());
            LazyMapTest {
                map,
                vector,
                lazy_maps,
            }
            .instantiate()
            .globalize()
        }

        pub fn new_lazy_map_into_map_then_get() -> ComponentAddress {
            let lazy_map = KeyValueStore::new();
            let lazy_maps = KeyValueStore::new();
            lazy_maps.insert("hello".to_owned(), lazy_map);
            let lazy_map = lazy_maps.get(&"hello".to_owned()).unwrap();
            lazy_map.insert("hello".to_owned(), "hello".to_owned());
            LazyMapTest {
                map: KeyValueStore::new(),
                vector: Vec::new(),
                lazy_maps,
            }
            .instantiate()
            .globalize()
        }

        pub fn new_lazy_map_with_get() -> ComponentAddress {
            let map = KeyValueStore::new();
            map.get(&"hello".to_owned());
            let lazy_maps = KeyValueStore::new();
            LazyMapTest {
                map,
                vector: Vec::new(),
                lazy_maps,
            }
            .instantiate()
            .globalize()
        }

        pub fn new_lazy_map_with_put() -> ComponentAddress {
            let map = KeyValueStore::new();
            map.insert("hello".to_owned(), "world".to_owned());
            let lazy_maps = KeyValueStore::new();
            LazyMapTest {
                map,
                vector: Vec::new(),
                lazy_maps,
            }
            .instantiate()
            .globalize()
        }

        pub fn overwrite_lazy_map(&mut self) -> () {
            self.lazy_maps
                .insert("hello".to_owned(), KeyValueStore::new())
        }

        pub fn clear_vector(&mut self) -> () {
            self.vector.clear()
        }
    }
}
