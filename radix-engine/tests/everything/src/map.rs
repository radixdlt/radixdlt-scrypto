use scrypto::constructs::Map;
use scrypto::*;

blueprint! {
    struct MapTest {}

    impl MapTest {
        pub fn test_map() -> Option<String> {
            let map = Map::new();
            map.insert("hello".to_owned(), "world".to_owned());
            map.get("hello")
        }
    }
}
