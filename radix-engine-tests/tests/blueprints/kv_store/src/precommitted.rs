use scrypto::prelude::*;

#[blueprint]
mod precommitted {
    struct Precommitted {
        store: KeyValueStore<u32, Vault>,
        deep_store: KeyValueStore<u32, KeyValueStore<u32, u32>>,
        deep_vault: KeyValueStore<u32, KeyValueStore<u32, Vault>>,
    }

    impl Precommitted {
        pub fn can_reference_precommitted_vault() -> ComponentAddress {
            let store = KeyValueStore::new();
            let bucket: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .mint_initial_supply(1);
            let vault = Vault::with_bucket(bucket);
            store.insert(0u32, vault);
            {
                let vault = store.get(&0u32).expect("Should be a vault");
                assert!(!vault.is_empty());
            }
            Precommitted {
                store,
                deep_store: KeyValueStore::new(),
                deep_vault: KeyValueStore::new(),
            }
            .instantiate()
            .globalize()
        }

        pub fn can_reference_deep_precommitted_value() -> ComponentAddress {
            let deep_store = KeyValueStore::new();
            let sub_store = KeyValueStore::new();
            sub_store.insert(0u32, 2u32);
            deep_store.insert(0u32, sub_store);

            let value: u32 = *deep_store.get(&0u32).unwrap().get(&0u32).unwrap();
            assert!(value == 2u32);

            Precommitted {
                store: KeyValueStore::new(),
                deep_store,
                deep_vault: KeyValueStore::new(),
            }
            .instantiate()
            .globalize()
        }

        pub fn can_reference_deep_precommitted_vault() -> ComponentAddress {
            let deep_vault = KeyValueStore::new();
            let sub_store = KeyValueStore::new();
            let bucket: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .mint_initial_supply(1);
            let vault = Vault::with_bucket(bucket);
            sub_store.insert(0u32, vault);
            deep_vault.insert(0u32, sub_store);

            {
                let store = deep_vault.get(&0u32).unwrap();
                let vault = store.get(&0u32).unwrap();
                assert!(!vault.is_empty());
            }

            Precommitted {
                store: KeyValueStore::new(),
                deep_store: KeyValueStore::new(),
                deep_vault,
            }
            .instantiate()
            .globalize()
        }
    }
}
