use scrypto::prelude::*;

#[blueprint]
mod basic {
    struct Basic {
        map: KeyValueStore<String, String>,
    }

    impl Basic {
        pub fn multiple_reads() -> ComponentAddress {
            let map = KeyValueStore::new();
            map.insert("hello".to_owned(), "hello".to_owned());
            map.insert("hello2".to_owned(), "hello2".to_owned());
            {
                let maybe_entry = map.get(&"hello".to_owned());
                let maybe_entry2 = map.get(&"hello2".to_owned());
                assert_eq!(*maybe_entry.unwrap(), "hello");
                assert_eq!(*maybe_entry2.unwrap(), "hello2");
            }
            Self { map }.instantiate().globalize()
        }

        pub fn remove_from_local() -> ComponentAddress {
            let map = KeyValueStore::new();
            map.insert("hello".to_owned(), "hello".to_owned());
            let removed = map.remove(&"hello".to_owned());
            assert_eq!(removed, Option::Some("hello".to_owned()));
            let maybe_entry = map.get(&"hello2".to_owned());
            assert!(maybe_entry.is_none());

            Self { map }.instantiate().globalize()
        }
    }
}
