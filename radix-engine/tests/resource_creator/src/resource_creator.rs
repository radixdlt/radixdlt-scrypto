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

        pub fn create_restricted_token(
            mint_auth: ResourceAddress,
            burn_auth: ResourceAddress,
            withdraw_auth: ResourceAddress,
            admin_auth: ResourceAddress,
        ) -> Bucket {
            ResourceBuilder::new_fungible()
                .divisibility(0)
                .mintable(
                    auth!(require(mint_auth)),
                    MUTABLE(auth!(require(admin_auth))),
                )
                .burnable(
                    auth!(require(burn_auth)),
                    MUTABLE(auth!(require(admin_auth))),
                )
                .restrict_withdraw(
                    auth!(require(withdraw_auth)),
                    MUTABLE(auth!(require(admin_auth))),
                )
                .initial_supply(5)
        }

        pub fn create_restricted_burn(badge_resource_address: ResourceAddress) -> Bucket {
            ResourceBuilder::new_fungible()
                .divisibility(0)
                .burnable(auth!(require(badge_resource_address)), LOCKED)
                .initial_supply(5)
        }

        pub fn set_mintable(resource_address: ResourceAddress, auth_address: ResourceAddress) {
            borrow_resource_manager!(resource_address).set_mintable(auth!(require(auth_address)));
        }

        pub fn set_burnable(resource_address: ResourceAddress, auth_address: ResourceAddress) {
            borrow_resource_manager!(resource_address).set_burnable(auth!(require(auth_address)));
        }

        pub fn set_withdrawable(resource_address: ResourceAddress, auth_address: ResourceAddress) {
            borrow_resource_manager!(resource_address)
                .set_withdrawable(auth!(require(auth_address)));
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

        pub fn create_fungible_fixed(amount: Decimal, divisibility: u8) -> Bucket {
            ResourceBuilder::new_fungible()
                .divisibility(divisibility)
                .metadata("name", "SUPER TOKEN")
                .initial_supply(amount)
        }
    }
}
