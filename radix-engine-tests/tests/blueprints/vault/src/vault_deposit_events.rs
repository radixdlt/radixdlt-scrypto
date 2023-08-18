use scrypto::prelude::*;

#[blueprint]
mod vault_events {
    struct ComponentWithVault {
        vault: Vault,
    }

    impl ComponentWithVault {
        pub fn create_vault_with_bucket() {
            Self {
                vault: Vault::with_bucket(
                    ResourceBuilder::new_fungible(OwnerRole::None)
                        .mint_initial_supply(5)
                        .into(),
                ),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();
        }
    }
}
