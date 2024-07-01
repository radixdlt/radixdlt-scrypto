use scrypto::prelude::*;

#[derive(Debug, PartialEq, Eq, ScryptoSbor, NonFungibleData)]
pub struct Sandwich {
    name: String,
}

#[blueprint]
mod big_vault {
    struct BigVault {
        vault: NonFungibleVault,
    }

    impl BigVault {
        pub fn new() {
            let resource_manager =
                ResourceBuilder::new_ruid_non_fungible::<Sandwich>(OwnerRole::None)
                    .mint_roles(mint_roles! {
                        minter => rule!(allow_all);
                        minter_updater => rule!(deny_all);
                    })
                    .create_with_no_initial_supply();

            let vault = resource_manager.create_empty_vault();

            Self { vault }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize();
        }

        pub fn mint(&mut self, count: usize) {
            let resource_manager = self.vault.resource_manager();
            for _ in 0..count {
                let bucket = resource_manager.mint_ruid_non_fungible(Sandwich {
                    name: "test".to_string(),
                });
                self.vault.put(bucket);
            }
        }

        pub fn non_fungibles(&mut self, count: u32) -> IndexSet<NonFungibleLocalId> {
            self.vault.non_fungible_local_ids(count)
        }
    }
}
