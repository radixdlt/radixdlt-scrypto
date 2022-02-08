use scrypto::prelude::*;

blueprint! {
    struct LazyMapTest {
        map: LazyMap<String, String>,
        vector: Vec<LazyMap<String, String>>
    }

    impl LazyMapTest {
        pub fn dangling_lazy_map() -> Option<String> {
            let map = LazyMap::new();
            map.insert("hello".to_owned(), "world".to_owned());
            map.get(&"hello".to_owned())
        }

        pub fn new_lazy_map_into_vector() -> Component {
            let map = LazyMap::new();
            map.get(&"hello".to_owned());
            let mut vector = Vec::new();
            vector.push(LazyMap::new());
            LazyMapTest { map, vector }.instantiate()
        }

        pub fn new_lazy_map_with_get() -> Component {
            let map = LazyMap::new();
            map.get(&"hello".to_owned());
            LazyMapTest { map, vector: Vec::new() }.instantiate()
        }

        pub fn new_lazy_map_with_put() -> Component {
            let map = LazyMap::new();
            map.insert("hello".to_owned(), "world".to_owned());
            LazyMapTest { map, vector: Vec::new() }.instantiate()
        }

        pub fn clear_vector(&mut self) -> () {
            self.vector.clear()
        }
    }
}
