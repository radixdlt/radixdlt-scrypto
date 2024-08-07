use scrypto::prelude::*;

#[derive(Debug, PartialEq, Eq, ScryptoSbor, NonFungibleData)]
pub struct Sandwich {
    pub name: String,
}

#[blueprint]
mod add_and_remove {
    struct AddAndRemove {
        vault: NonFungibleVault,
        other_vault: NonFungibleVault,
    }

    impl AddAndRemove {
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
            let other_vault = resource_manager.create_empty_vault();

            Self { vault, other_vault }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize();
        }

        pub fn add_and_remove(&mut self) {
            let resource_manager = self.vault.resource_manager();

            let id = NonFungibleLocalId::integer(1);

            let bucket = resource_manager.mint_non_fungible(
                &id,
                Sandwich {
                    name: "Test".to_owned(),
                },
            );
            self.vault.put(bucket);
            let bucket = self.vault.take_non_fungible(&id);
            self.other_vault.put(bucket);
        }
    }
}
