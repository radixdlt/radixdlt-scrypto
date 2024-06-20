use scrypto::prelude::*;

#[derive(Debug, PartialEq, Eq, ScryptoSbor, NonFungibleData)]
pub struct Sandwich {
    pub name: String,
}

#[blueprint]
mod mint_and_burn {
    struct MintAndBurn {
        vault: NonFungibleVault,
    }

    impl MintAndBurn {
        pub fn new() {
            let resource_manager =
                ResourceBuilder::new_integer_non_fungible::<Sandwich>(OwnerRole::None)
                    .mint_roles(mint_roles! {
                        minter => rule!(allow_all);
                        minter_updater => rule!(deny_all);
                    })
                    .burn_roles(burn_roles! {
                        burner => rule!(allow_all);
                        burner_updater => rule!(deny_all);
                    })
                    .create_with_no_initial_supply();

            let vault = resource_manager.create_empty_vault();

            Self { vault }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize();
        }

        pub fn mint_and_burn(&mut self) {
            let resource_manager = self.vault.resource_manager();

            let id = NonFungibleLocalId::integer(1);

            let bucket = resource_manager.mint_non_fungible(
                &id,
                Sandwich {
                    name: "Test".to_owned(),
                },
            );
            bucket.burn();
        }

        pub fn mint_and_burn_2x(&mut self) {
            let resource_manager = self.vault.resource_manager();

            let id = NonFungibleLocalId::integer(1);

            let bucket = resource_manager.mint_non_fungible(
                &id,
                Sandwich {
                    name: "Test".to_owned(),
                },
            );
            bucket.burn();

            let bucket = resource_manager.mint_non_fungible(
                &id,
                Sandwich {
                    name: "Test".to_owned(),
                },
            );
            bucket.burn();
        }
    }
}
