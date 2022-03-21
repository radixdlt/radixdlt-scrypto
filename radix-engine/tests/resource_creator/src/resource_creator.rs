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
            ResourceBuilder::new_fungible(0)
                .flags(RESTRICTED_TRANSFER)
                .badge(badge_resource_def_id, MAY_TRANSFER)
                .initial_supply_fungible(5)
        }

        pub fn create_restricted_mint(badge_resource_def_id: ResourceDefId) -> Bucket {
            ResourceBuilder::new_fungible(0)
                .flags(MINTABLE)
                .badge(badge_resource_def_id, MAY_MINT)
                .initial_supply_fungible(5)
        }

        pub fn create_restricted_burn(badge_resource_def_id: ResourceDefId) -> Bucket {
            ResourceBuilder::new_fungible(0)
                .flags(BURNABLE)
                .badge(badge_resource_def_id, MAY_BURN)
                .initial_supply_fungible(5)
        }

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
