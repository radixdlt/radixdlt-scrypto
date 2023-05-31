use scrypto::prelude::*;

#[blueprint]
mod blueprint {
    struct NonFungibleVault {
        vault: Vault,
    }

    impl NonFungibleVault {
        pub fn new(tokens: Bucket) -> Global<NonFungibleVault> {
            Self {
                vault: Vault::with_bucket(tokens),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }
    }
}
