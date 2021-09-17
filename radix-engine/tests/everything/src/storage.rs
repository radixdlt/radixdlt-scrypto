use scrypto::constructs::Storage;
use scrypto::*;

blueprint! {
    struct StorageTest {}

    impl StorageTest {
        pub fn test_storage() -> Option<String> {
            let s = Storage::new();
            s.insert("hello".to_owned(), "world".to_owned());
            s.get("hello")
        }
    }
}
