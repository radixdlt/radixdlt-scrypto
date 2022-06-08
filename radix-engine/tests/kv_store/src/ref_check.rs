use scrypto::prelude::*;

blueprint! {
    struct RefCheck {
        store: KeyValueStore<u32, Vault>,
    }

    impl RefCheck {
        pub fn cannot_directly_reference_inserted_vault() -> ComponentAddress {
            let store = KeyValueStore::new();
            let bucket: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .initial_supply(1);
            let vault = Vault::with_bucket(bucket);
            let vault_id = vault.0.clone();
            store.insert(0u32, vault);

            let vault = Vault(vault_id);
            vault.is_empty();

            RefCheck {
                store,
            }
            .instantiate()
            .globalize()
        }
    }
}
