use scrypto::prelude::*;

#[derive(NonFungibleData)]
pub struct Sandwich {
    pub name: String,
    #[scrypto(mutable)]
    pub available: bool,
}

blueprint! {
    struct ResourceCreator {}

    impl ResourceCreator {
        pub fn create_restricted_transfer(badge_resource_def_id: ResourceDefId) -> Bucket {
            ResourceBuilder::new_fungible()
                .divisibility(0)
                .auth("take_from_vault", auth!(require(badge_resource_def_id)))
                .initial_supply(5)
        }

        pub fn create_restricted_mint(badge_resource_def_id: ResourceDefId) -> Bucket {
            ResourceBuilder::new_fungible()
                .divisibility(0)
                .auth("take_from_vault", auth!(allow_all))
                .auth("mint", auth!(require(badge_resource_def_id)))
                .initial_supply(5)
        }

        pub fn create_restricted_burn(badge_resource_def_id: ResourceDefId) -> Bucket {
            ResourceBuilder::new_fungible()
                .divisibility(0)
                .auth("take_from_vault", auth!(allow_all))
                .auth("burn", auth!(require(badge_resource_def_id)))
                .initial_supply(5)
        }

        pub fn create_non_fungible_fixed() -> Bucket {
            ResourceBuilder::new_non_fungible()
                .auth("take_from_vault", auth!(allow_all))
                .metadata("name", "Katz's Sandwiches")
                .initial_supply([
                    (
                        NonFungibleId::from(1u128),
                        Sandwich {
                            name: "One".to_owned(),
                            available: true,
                        },
                    ),
                    (
                        NonFungibleId::from(2u128),
                        Sandwich {
                            name: "Two".to_owned(),
                            available: true,
                        },
                    ),
                    (
                        NonFungibleId::from(3u128),
                        Sandwich {
                            name: "Three".to_owned(),
                            available: true,
                        },
                    ),
                ])
        }

        pub fn create_fungible_fixed(amount: Decimal, divisibility: u8) -> Bucket {
            ResourceBuilder::new_fungible()
                .divisibility(divisibility)
                .auth("take_from_vault", auth!(allow_all))
                .metadata("name", "SUPER TOKEN")
                .initial_supply(amount)
        }
    }
}
