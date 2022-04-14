use scrypto::prelude::*;

blueprint! {
    struct LazyMapTest {
        map: LazyMap<String, String>,
        vector: Vec<LazyMap<String, String>>,
        lazy_maps: LazyMap<String, LazyMap<String, String>>,
    }

    impl LazyMapTest {
        pub fn dangling_lazy_map() -> Option<String> {
            let map = LazyMap::new();
            map.insert("hello".to_owned(), "world".to_owned());
            map.get(&"hello".to_owned())
        }

        pub fn new_lazy_map_into_vector() -> ComponentAddress {
            let map = LazyMap::new();
            map.get(&"hello".to_owned());
            let mut vector = Vec::new();
            vector.push(LazyMap::new());
            let lazy_maps = LazyMap::new();
            LazyMapTest {
                map,
                vector,
                lazy_maps,
            }
            .instantiate()
            .auth(component_authorization! {
                "clear_vector" => auth!(allow_all),
                "dangling_lazy_map" => auth!(allow_all),
                "overwrite_lazy_map" => auth!(allow_all)
            })
            .globalize()
        }

        pub fn new_lazy_map_into_lazy_map() -> ComponentAddress {
            let map = LazyMap::new();
            let vector = Vec::new();
            let lazy_maps = LazyMap::new();
            lazy_maps.insert("hello".to_owned(), LazyMap::new());
            LazyMapTest {
                map,
                vector,
                lazy_maps,
            }
            .instantiate()
            .auth(component_authorization! {
                "clear_vector" => auth!(allow_all),
                "dangling_lazy_map" => auth!(allow_all),
                "overwrite_lazy_map" => auth!(allow_all)
            })
            .globalize()
        }

        pub fn new_lazy_map_into_map_then_get() -> ComponentAddress {
            let lazy_map = LazyMap::new();
            let lazy_maps = LazyMap::new();
            lazy_maps.insert("hello".to_owned(), lazy_map);
            let lazy_map = lazy_maps.get(&"hello".to_owned()).unwrap();
            lazy_map.insert("hello".to_owned(), "hello".to_owned());
            LazyMapTest {
                map: LazyMap::new(),
                vector: Vec::new(),
                lazy_maps,
            }
            .instantiate()
            .auth(component_authorization! {
                "clear_vector" => auth!(allow_all),
                "dangling_lazy_map" => auth!(allow_all),
                "overwrite_lazy_map" => auth!(allow_all)
            })
            .globalize()
        }

        pub fn new_lazy_map_with_get() -> ComponentAddress {
            let map = LazyMap::new();
            map.get(&"hello".to_owned());
            let lazy_maps = LazyMap::new();
            LazyMapTest {
                map,
                vector: Vec::new(),
                lazy_maps,
            }
            .instantiate()
            .auth(component_authorization! {
                "clear_vector" => auth!(allow_all),
                "dangling_lazy_map" => auth!(allow_all),
                "overwrite_lazy_map" => auth!(allow_all)
            })
            .globalize()
        }

        pub fn new_lazy_map_with_put() -> ComponentAddress {
            let map = LazyMap::new();
            map.insert("hello".to_owned(), "world".to_owned());
            let lazy_maps = LazyMap::new();
            LazyMapTest {
                map,
                vector: Vec::new(),
                lazy_maps,
            }
            .instantiate()
            .auth(component_authorization! {
                "clear_vector" => auth!(allow_all),
                "dangling_lazy_map" => auth!(allow_all),
                "overwrite_lazy_map" => auth!(allow_all)
            })
            .globalize()
        }

        pub fn overwrite_lazy_map(&mut self) -> () {
            self.lazy_maps.insert("hello".to_owned(), LazyMap::new())
        }

        pub fn clear_vector(&mut self) -> () {
            self.vector.clear()
        }
    }
}
