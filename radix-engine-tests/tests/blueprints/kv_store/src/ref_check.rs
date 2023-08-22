use scrypto::prelude::*;

#[blueprint]
mod ref_check {
    struct RefCheck {
        store: KeyValueStore<u32, Vault>,
        store_store: KeyValueStore<u32, KeyValueStore<u32, Vault>>,
    }

    impl RefCheck {
        pub fn cannot_directly_reference_inserted_vault() -> Global<RefCheck> {
            let store = KeyValueStore::new();
            let bucket: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata(metadata! {
                    init {
                        "name" => "TestToken".to_owned(), locked;
                    }
                })
                .mint_initial_supply(1)
                .into();
            let vault = Vault::with_bucket(bucket);
            let vault_id = vault.0.clone();
            store.insert(0u32, vault);

            let vault = Vault(vault_id);
            vault.is_empty();

            RefCheck {
                store,
                store_store: KeyValueStore::new(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn cannot_directly_reference_vault_after_container_moved() -> Global<RefCheck> {
            let store = KeyValueStore::new();
            let bucket: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata(metadata! {
                    init {
                        "name" => "TestToken".to_owned(), locked;
                    }
                })
                .mint_initial_supply(1)
                .into();
            let vault = Vault::with_bucket(bucket);
            let vault_id = vault.0.clone();
            store.insert(0u32, vault);
            {
                let _vault = store.get(&0u32).expect("Should be a vault");
            }
            let store_store = KeyValueStore::new();
            store_store.insert(0u32, store);

            let vault = Vault(vault_id);
            vault.is_empty();

            RefCheck {
                store: KeyValueStore::new(),
                store_store,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn cannot_directly_reference_vault_after_container_stored() -> bool {
            let store = KeyValueStore::new();
            let bucket: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata(metadata! {
                    init {
                        "name" => "TestToken".to_owned(), locked;
                    }
                })
                .mint_initial_supply(1)
                .into();
            let vault = Vault::with_bucket(bucket);
            let vault_id = vault.0.clone();
            store.insert(0u32, vault);

            RefCheck {
                store,
                store_store: KeyValueStore::new(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();

            let vault = Vault(vault_id);
            vault.is_empty()
        }
    }
}
