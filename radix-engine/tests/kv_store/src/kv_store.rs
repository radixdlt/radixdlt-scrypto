use scrypto::prelude::*;

blueprint! {
    struct KeyValueStoreTest {
        map: KeyValueStore<String, String>,
        vector: Vec<KeyValueStore<String, String>>,
        key_value_stores: KeyValueStore<String, KeyValueStore<String, String>>,
    }

    impl KeyValueStoreTest {
        pub fn new_key_value_store_into_vector() -> ComponentAddress {
            let map = KeyValueStore::new();
            map.get(&"hello".to_owned());
            let mut vector = Vec::new();
            vector.push(KeyValueStore::new());
            let key_value_stores = KeyValueStore::new();
            KeyValueStoreTest {
                map,
                vector,
                key_value_stores,
            }
            .instantiate()
            .globalize()
        }

        pub fn new_key_value_store_into_key_value_store() -> ComponentAddress {
            let map = KeyValueStore::new();
            let vector = Vec::new();
            let key_value_stores = KeyValueStore::new();
            key_value_stores.insert("hello".to_owned(), KeyValueStore::new());
            KeyValueStoreTest {
                map,
                vector,
                key_value_stores,
            }
            .instantiate()
            .globalize()
        }

        pub fn new_key_value_store_into_map_then_get() -> ComponentAddress {
            let key_value_store = KeyValueStore::new();
            let key_value_stores = KeyValueStore::new();
            key_value_stores.insert("hello".to_owned(), key_value_store);
            let key_value_store = key_value_stores.get(&"hello".to_owned()).unwrap();
            key_value_store.insert("hello".to_owned(), "hello".to_owned());
            KeyValueStoreTest {
                map: KeyValueStore::new(),
                vector: Vec::new(),
                key_value_stores,
            }
            .instantiate()
            .globalize()
        }

        pub fn new_key_value_store_with_get() -> ComponentAddress {
            let map = KeyValueStore::new();
            map.get(&"hello".to_owned());
            let key_value_stores = KeyValueStore::new();
            KeyValueStoreTest {
                map,
                vector: Vec::new(),
                key_value_stores,
            }
            .instantiate()
            .globalize()
        }

        pub fn new_key_value_store_with_put() -> ComponentAddress {
            let map = KeyValueStore::new();
            map.insert("hello".to_owned(), "world".to_owned());
            let key_value_stores = KeyValueStore::new();
            KeyValueStoreTest {
                map,
                vector: Vec::new(),
                key_value_stores,
            }
            .instantiate()
            .globalize()
        }

        pub fn overwrite_key_value_store(&mut self) -> () {
            self.key_value_stores
                .insert("hello".to_owned(), KeyValueStore::new())
        }

        pub fn clear_vector(&mut self) -> () {
            self.vector.clear()
        }
    }
}
