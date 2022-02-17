use scrypto::prelude::*;

blueprint! {
    struct NonExistentVault {
        vault: Vid
    }

     impl NonExistentVault {
        pub fn create_non_existent_vault() -> Component {
            NonExistentVault {
                vault: Vid(Context::transaction_hash(), 1025)
            }.instantiate()
        }
    }
}