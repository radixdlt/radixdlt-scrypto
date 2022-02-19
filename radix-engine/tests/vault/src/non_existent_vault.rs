use scrypto::prelude::*;

blueprint! {
    struct NonExistentVault {
        vault: Option<Vid>,
        vaults: LazyMap<u128, Vid>,
    }

     impl NonExistentVault {
        pub fn create_component_with_non_existent_vault() -> Component {
            NonExistentVault {
                vault: Option::Some(Vid(Context::transaction_hash(), 1025)),
                vaults: LazyMap::new(),
            }.instantiate()
        }

        pub fn new() -> Component {
            NonExistentVault {
                vault: Option::None,
                vaults: LazyMap::new(),
            }.instantiate()
        }

        pub fn create_non_existent_vault(&mut self) {
            self.vault = Option::Some(Vid(Context::transaction_hash(), 1025))
        }

        pub fn create_lazy_map_with_non_existent_vault() -> Component {
            let vaults = LazyMap::new();
            vaults.insert(0, Vid(Context::transaction_hash(), 1025));
            NonExistentVault {
                vault: Option::None,
                vaults,
            }.instantiate()
        }

        pub fn create_non_existent_vault_in_lazy_map(&mut self) {
            self.vaults.insert(0, Vid(Context::transaction_hash(), 1025));
        }
    }
}