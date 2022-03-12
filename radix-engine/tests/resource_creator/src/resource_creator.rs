use scrypto::prelude::*;

#[derive(NonFungibleData)]
pub struct Sandwich {
    pub name: String,
    #[scrypto(mutable)]
    pub available: bool,
}

blueprint! {
    struct ResourceCreator {
    }

    impl ResourceCreator {
        pub fn create_non_fungible_fixed() -> Bucket {
            ResourceBuilder::new_non_fungible()
                .metadata("name", "Katz's Sandwiches")
                .initial_supply_non_fungible([
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
    }
}
