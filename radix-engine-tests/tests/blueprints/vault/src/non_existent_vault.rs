use scrypto::prelude::*;

#[blueprint]
mod non_existent_vault {
    struct NonExistentVault {
        vault: Option<Vault>,
        vaults: KeyValueStore<u128, Vault>,
    }

    impl NonExistentVault {
        pub fn create_component_with_non_existent_vault() -> ComponentAddress {
            NonExistentVault {
                vault: Option::Some(Vault(Own(NodeId([1u8; 27])))),
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
            self.vault = Option::Some(Vault(Own(NodeId([1u8; 27]))))
        }

        pub fn create_kv_store_with_non_existent_vault() -> ComponentAddress {
            let vaults = KeyValueStore::new();
            vaults.insert(0, Vault(Own(NodeId([1u8; 27]))));
            NonExistentVault {
                vault: Option::None,
                vaults,
            }
            .instantiate()
            .globalize()
        }

        pub fn create_non_existent_vault_in_kv_store(&mut self) {
            self.vaults.insert(0, Vault(Own(NodeId([1u8; 27]))));
        }
    }
}
