use scrypto::prelude::*;

#[derive(Debug, PartialEq, Eq, ScryptoSbor, NonFungibleData)]
pub struct Sandwich {
    pub name: String,
}

#[blueprint]
mod mint_and_burn {
    struct MintAndBurn {
        vault: Vault,
        other_vault: Vault,
    }

    impl MintAndBurn {
        pub fn new() {
            let resource_manager = ResourceBuilder::new_integer_non_fungible::<Sandwich>()
                .mintable(rule!(allow_all), rule!(deny_all))
                .burnable(rule!(allow_all), rule!(deny_all))
                .create_with_no_initial_supply();

            let vault = resource_manager.create_empty_vault();
            let other_vault = resource_manager.create_empty_vault();

            Self { vault, other_vault }.instantiate().globalize();
        }

        pub fn mint_and_burn(&mut self, i: u64) {
            let resource_manager = self.vault.resource_manager();

            let bucket = resource_manager.mint_non_fungible(
                &NonFungibleLocalId::integer(i),
                Sandwich {
                    name: "Test".to_owned(),
                },
            );
            self.vault.put(bucket);
            let bucket = self.vault.take_all();
            self.other_vault.put(bucket);
        }
    }
}