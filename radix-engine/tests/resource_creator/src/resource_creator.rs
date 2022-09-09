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
        pub fn set_mintable(resource_address: ResourceAddress, auth_address: ResourceAddress) {
            borrow_resource_manager!(resource_address).set_mintable(rule!(require(auth_address)));
        }

        pub fn set_burnable(resource_address: ResourceAddress, auth_address: ResourceAddress) {
            borrow_resource_manager!(resource_address).set_burnable(rule!(require(auth_address)));
        }

        pub fn set_withdrawable(resource_address: ResourceAddress, auth_address: ResourceAddress) {
            borrow_resource_manager!(resource_address)
                .set_withdrawable(rule!(require(auth_address)));
        }

        pub fn set_depositable(resource_address: ResourceAddress, auth_address: ResourceAddress) {
            borrow_resource_manager!(resource_address)
                .set_depositable(rule!(require(auth_address)));
        }

        pub fn lock_mintable(resource_address: ResourceAddress) {
            borrow_resource_manager!(resource_address).lock_mintable();
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
    }
}
