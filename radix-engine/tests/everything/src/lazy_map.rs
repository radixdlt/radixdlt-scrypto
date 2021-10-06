use scrypto::core::LazyMap;
use scrypto::*;

blueprint! {
    struct LazyMapTest {}

    impl LazyMapTest {
        pub fn test_lazy_map() -> Option<String> {
            let s = LazyMap::new();
            s.insert("hello".to_owned(), "world".to_owned());
            s.get("hello")
        }
    }
}
