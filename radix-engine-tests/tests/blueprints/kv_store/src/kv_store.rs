use scrypto::prelude::*;

#[blueprint]
mod key_value_store_test {
    struct KeyValueStoreTest {
        map: KeyValueStore<String, String>,
        vector: Vec<KeyValueStore<String, String>>,
        kv_stores: KeyValueStore<String, KeyValueStore<String, String>>,
    }

    impl KeyValueStoreTest {
        pub fn new_kv_store_into_vector() -> ComponentAddress {
            let map = KeyValueStore::new();
            map.get(&"hello".to_owned());
            let mut vector = Vec::new();
            vector.push(KeyValueStore::new());
            let kv_stores = KeyValueStore::new();
            KeyValueStoreTest {
                map,
                vector,
                kv_stores,
            }
            .instantiate()
            .globalize()
        }

        pub fn new_kv_store_into_kv_store() -> ComponentAddress {
            let map = KeyValueStore::new();
            let vector = Vec::new();
            let kv_stores = KeyValueStore::new();
            kv_stores.insert("hello".to_owned(), KeyValueStore::new());
            KeyValueStoreTest {
                map,
                vector,
                kv_stores,
            }
            .instantiate()
            .globalize()
        }

        pub fn new_kv_store_into_map_then_get() -> ComponentAddress {
            let kv_store = KeyValueStore::new();
            let kv_stores = KeyValueStore::new();
            kv_stores.insert("hello".to_owned(), kv_store);
            {
                let kv_store = kv_stores.get(&"hello".to_owned()).unwrap();
                kv_store.insert("hello".to_owned(), "hello".to_owned());
            }
            KeyValueStoreTest {
                map: KeyValueStore::new(),
                vector: Vec::new(),
                kv_stores,
            }
            .instantiate()
            .globalize()
        }

        pub fn new_kv_store_with_get() -> ComponentAddress {
            let map = KeyValueStore::new();
            map.get(&"hello".to_owned());
            let kv_stores = KeyValueStore::new();
            KeyValueStoreTest {
                map,
                vector: Vec::new(),
                kv_stores,
            }
            .instantiate()
            .globalize()
        }

        pub fn new_kv_store_with_put() -> ComponentAddress {
            let map = KeyValueStore::new();
            map.insert("hello".to_owned(), "world".to_owned());
            let kv_stores = KeyValueStore::new();
            KeyValueStoreTest {
                map,
                vector: Vec::new(),
                kv_stores,
            }
            .instantiate()
            .globalize()
        }

        pub fn overwrite_kv_store(&mut self) -> () {
            self.kv_stores
                .insert("hello".to_owned(), KeyValueStore::new())
        }

        pub fn clear_vector(&mut self) -> () {
            self.vector.clear()
        }
    }
}
