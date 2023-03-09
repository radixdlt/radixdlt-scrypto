use scrypto::prelude::*;

#[blueprint]
mod blueprint {
    struct NonFungibleVault {
        vault: Vault,
    }

    impl NonFungibleVault {
        pub fn new(tokens: Bucket) -> ComponentAddress {
            Self {
                vault: Vault::with_bucket(tokens),
            }
            .instantiate()
            .globalize()
        }
    }
}
