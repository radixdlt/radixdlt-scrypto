use scrypto::prelude::*;

#[derive(Debug, PartialEq, Eq, ScryptoSbor, NonFungibleData)]
pub struct Sandwich {
    pub name: String,
}

#[blueprint]
mod mint_and_burn {
    struct MintAndBurn {
        vault: Vault,
    }

    impl MintAndBurn {
        pub fn new() {
            let resource_manager = ResourceBuilder::new_integer_non_fungible::<Sandwich>()
                .mintable(rule!(allow_all), rule!(deny_all))
                .burnable(rule!(allow_all), rule!(deny_all))
                .create_with_no_initial_supply();

            let bucket = resource_manager.mint_non_fungible(
                &NonFungibleLocalId::integer(0),
                Sandwich {
                    name: "Test".to_owned(),
                },
            );

            let vault = Vault::with_bucket(bucket);

            Self { vault }.instantiate().globalize();
        }

        pub fn mint_and_burn(&mut self) {
            self.vault.take_all().burn();

            let resource_manager = self.vault.resource_manager();

            let bucket = resource_manager.mint_non_fungible(
                &NonFungibleLocalId::integer(0),
                Sandwich {
                    name: "Test".to_owned(),
                },
            );
            self.vault.put(bucket);
            self.vault.take_all().burn();
        }
    }
}