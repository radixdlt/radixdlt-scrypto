use scrypto::prelude::*;

#[blueprint]
mod basic {
    struct Basic {
        map: KeyValueStore<String, String>,
    }

    impl Basic {
        pub fn new() -> Global<Basic> {
            let map = KeyValueStore::new();
            Self { map }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn new_with_entry(key: String, value: String) -> Global<Basic> {
            let map = KeyValueStore::new();
            map.insert(key, value);
            Self { map }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn multiple_reads() -> Global<Basic> {
            let map = KeyValueStore::new();
            map.insert("hello".to_owned(), "hello".to_owned());
            map.insert("hello2".to_owned(), "hello2".to_owned());
            {
                let maybe_entry = map.get(&"hello".to_owned());
                let maybe_entry2 = map.get(&"hello2".to_owned());
                assert_eq!(*maybe_entry.unwrap(), "hello");
                assert_eq!(*maybe_entry2.unwrap(), "hello2");
            }
            Self { map }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn remove_from_local() -> Global<Basic> {
            let map = KeyValueStore::new();
            map.insert("hello".to_owned(), "hello".to_owned());
            let removed = map.remove(&"hello".to_owned());
            assert_eq!(removed, Option::Some("hello".to_owned()));
            {
                let maybe_entry = map.get(&"hello2".to_owned());
                assert!(maybe_entry.is_none());
            }

            Self { map }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn insert(&mut self, key: String, value: String) {
            self.map.insert(key, value);
        }

        pub fn remove(&mut self, key: String) -> Option<String> {
            self.map.remove(&key)
        }
    }
}

#[blueprint]
mod kv_vault {
    struct KVVault {
        map: KeyValueStore<String, Vault>,
    }

    impl KVVault {
        fn new_fungible() -> Bucket {
            ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata(metadata! {
                    init {
                        "name" => "TestToken".to_owned(), locked;
                    }
                })
                .mint_initial_supply(1)
                .into()
        }

        pub fn new() -> Global<KVVault> {
            let bucket = Self::new_fungible();
            let vault = Vault::with_bucket(bucket);
            let map = KeyValueStore::new();
            map.insert("key".to_string(), vault);
            Self { map }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn remove(&mut self, key: String) -> Option<Vault> {
            self.map.remove(&key)
        }
    }
}
