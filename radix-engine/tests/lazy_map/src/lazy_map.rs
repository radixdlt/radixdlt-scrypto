use scrypto::prelude::*;

blueprint! {
    struct LazyMapTest {
        map: LazyMap<String, String>
    }

    impl LazyMapTest {
        pub fn dangling_lazy_map() -> Option<String> {
            let map = LazyMap::new();
            map.insert("hello".to_owned(), "world".to_owned());
            map.get(&"hello".to_owned())
        }

        pub fn new_lazy_map_with_get() -> Component {
            let map = LazyMap::new();
            map.get(&"hello".to_owned());
            LazyMapTest { map }.instantiate()
        }

        pub fn new_lazy_map_with_put() -> Component {
            let map = LazyMap::new();
            map.insert("hello".to_owned(), "world".to_owned());
            LazyMapTest { map }.instantiate()
        }
    }
}
