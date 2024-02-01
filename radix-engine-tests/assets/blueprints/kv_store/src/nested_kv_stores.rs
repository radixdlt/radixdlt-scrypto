use scrypto::prelude::*;

#[blueprint]
mod nested_kv_stores {
    struct NestedKvStores {
        counters: KeyValueStore<String, KeyValueStore<String, u64>>,
    }

    impl NestedKvStores {
        pub fn instantiate() -> Global<NestedKvStores> {
            let mut counters = KeyValueStore::<String, KeyValueStore<String, u64>>::new();
            counters.insert("A".into(), {
                let kv_store = KeyValueStore::new();
                kv_store.insert("A1".into(), 0);
                kv_store
            });

            {
                let mut inner_map = counters.get_mut(&String::from("A")).unwrap();
                let mut value = inner_map.get_mut(&"A1".into()).unwrap();
                *value += 1;
            }

            Self { counters }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }
    }
}
