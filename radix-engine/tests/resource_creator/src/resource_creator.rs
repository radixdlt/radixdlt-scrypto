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
        pub fn create_restricted_transfer(badge_resource_address: ResourceAddress) -> Bucket {
            ResourceBuilder::new_fungible()
                .divisibility(0)
                .restrict_withdraw(auth!(require(badge_resource_address)), LOCKED)
                .initial_supply(5)
        }

        pub fn create_restricted_mint(badge_resource_address: ResourceAddress) -> Bucket {
            ResourceBuilder::new_fungible()
                .divisibility(0)
                .mintable(auth!(require(badge_resource_address)), LOCKED)
                .initial_supply(5)
        }

        pub fn create_restricted_burn(badge_resource_address: ResourceAddress) -> Bucket {
            ResourceBuilder::new_fungible()
                .divisibility(0)
                .burnable(auth!(require(badge_resource_address)), LOCKED)
                .initial_supply(5)
        }

        pub fn create_non_fungible_fixed() -> Bucket {
            ResourceBuilder::new_non_fungible()
                .metadata("name", "Katz's Sandwiches")
                .initial_supply([
                    (
                        NonFungibleId::from_u32(1),
                        Sandwich {
                            name: "One".to_owned(),
                            available: true,
                        },
                    ),
                    (
                        NonFungibleId::from_u32(2),
                        Sandwich {
                            name: "Two".to_owned(),
                            available: true,
                        },
                    ),
                    (
                        NonFungibleId::from_u32(3),
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
                .metadata("name", "SUPER TOKEN")
                .initial_supply(amount)
        }
    }
}
