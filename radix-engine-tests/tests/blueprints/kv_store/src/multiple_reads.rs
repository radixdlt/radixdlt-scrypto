use scrypto::prelude::*;

#[blueprint]
mod multiple_reads {
    struct MultipleReads {
        map: KeyValueStore<String, String>,
    }

    impl MultipleReads {
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
            MultipleReads { map }.instantiate().globalize()
        }
    }
}
