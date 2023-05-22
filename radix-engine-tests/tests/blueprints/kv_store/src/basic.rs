use scrypto::prelude::*;

#[blueprint]
mod basic {
    struct Basic {
        map: KeyValueStore<String, String>,
    }

    impl Basic {
        pub fn new() -> ComponentAddress {
            let map = KeyValueStore::new();
            Self { map }.instantiate().globalize()
        }

        pub fn new_with_entry(key: String, value: String) -> ComponentAddress {
            let map = KeyValueStore::new();
            map.insert(key, value);
            Self { map }.instantiate().globalize()
        }

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
            {
                let maybe_entry = map.get(&"hello2".to_owned());
                assert!(maybe_entry.is_none());
            }

            Self { map }.instantiate().globalize()
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
            ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .mint_initial_supply(1)
        }

        pub fn new() -> ComponentAddress {
            let bucket = Self::new_fungible();
            let vault = Vault::with_bucket(bucket);
            let map = KeyValueStore::new();
            map.insert("key".to_string(), vault);
            Self { map }.instantiate().globalize()
        }

        pub fn remove(&mut self, key: String) -> Option<Vault> {
            self.map.remove(&key)
        }
    }
}

#[blueprint]
mod db {
    struct DatabaseBench {
        map: KeyValueStore<u32, String>,
    }

    impl DatabaseBench {
        pub fn new() -> ComponentAddress {
            let map = KeyValueStore::new();
            Self { map }.instantiate().globalize()
        }

        pub fn insert(&mut self, len: u32) {
            let val = (0..len).map(|_| 'A').collect::<String>();
            self.map.insert(len, val);
        }

        pub fn read(&mut self, key: u32) -> String {
            (*self.map.get(&key).unwrap().clone()).to_string()
        }

        pub fn read_repeat(&mut self, len: u32, count: u32) -> bool {
            for _ in 0..count {
                if self.map.get(&len).is_none() {
                    return false;
                }
            }
            true
        }

    }
}