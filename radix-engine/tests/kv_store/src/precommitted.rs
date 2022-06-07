use scrypto::prelude::*;

blueprint! {
    struct Precommitted {
        store: KeyValueStore<u32, Vault>,
    }

    impl Precommitted {
        pub fn can_reference_precommitted_vault() -> ComponentAddress {
            let store = KeyValueStore::new();
            let bucket: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .initial_supply(1);
            let vault = Vault::with_bucket(bucket);
            store.insert(0u32, vault);
            let vault = store.get(&0u32).expect("Should be a vault");
            assert!(!vault.is_empty());
            Precommitted { store }.instantiate().globalize()
        }
    }
}
