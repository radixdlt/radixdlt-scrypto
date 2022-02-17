use scrypto::prelude::*;

blueprint! {
    struct NonExistentVault {
        vault: Option<Vid>
    }

     impl NonExistentVault {
        pub fn create_component_with_non_existent_vault() -> Component {
            NonExistentVault {
                vault: Option::Some(Vid(Context::transaction_hash(), 1025))
            }.instantiate()
        }

        pub fn new() -> Component {
            NonExistentVault {
                vault: Option::None
            }.instantiate()
        }

        pub fn create_non_existent_vault(&mut self) {
            self.vault = Option::Some(Vid(Context::transaction_hash(), 1025))
        }
    }
}