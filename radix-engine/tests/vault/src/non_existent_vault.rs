use scrypto::prelude::*;

blueprint! {
    struct NonExistentVault {
        vault: Option<Vault>,
        vaults: KeyValueStore<u128, Vault>,
    }

    impl NonExistentVault {
        pub fn create_component_with_non_existent_vault() -> ComponentAddress {
            NonExistentVault {
                vault: Option::Some(Vault((Runtime::transaction_hash(), 1025))),
                vaults: KeyValueStore::new(),
            }
            .instantiate()
            .globalize()
        }

        pub fn new() -> ComponentAddress {
            NonExistentVault {
                vault: Option::None,
                vaults: KeyValueStore::new(),
            }
            .instantiate()
            .globalize()
        }

        pub fn create_non_existent_vault(&mut self) {
            self.vault = Option::Some(Vault((Runtime::transaction_hash(), 1025)))
        }

        pub fn create_lazy_map_with_non_existent_vault() -> ComponentAddress {
            let vaults = KeyValueStore::new();
            vaults.insert(0, Vault((Runtime::transaction_hash(), 1025)));
            NonExistentVault {
                vault: Option::None,
                vaults,
            }
            .instantiate()
            .globalize()
        }

        pub fn create_non_existent_vault_in_lazy_map(&mut self) {
            self.vaults
                .insert(0, Vault((Runtime::transaction_hash(), 1025)));
        }
    }
}
