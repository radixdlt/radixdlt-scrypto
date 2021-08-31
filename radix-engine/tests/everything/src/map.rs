use scrypto::constructs::Map;
use scrypto::*;

blueprint! {
    struct MapTest {}

    impl MapTest {
        pub fn test_map() -> Option<String> {
            let map = Map::new();
            map.put_entry("hello", "world");
            map.get_entry("hello")
        }
    }
}
