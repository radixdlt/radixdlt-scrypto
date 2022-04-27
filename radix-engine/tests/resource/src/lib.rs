use scrypto::prelude::*;

blueprint! {
    struct ResourceTest;

    impl ResourceTest {
        pub fn create_fungible() -> (Bucket, ResourceAddress) {
            let badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply(1);
            let token_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .mintable(rule!(require(badge.resource_address())), LOCKED)
                .burnable(rule!(require(badge.resource_address())), LOCKED)
                .no_initial_supply();
            (badge, token_address)
        }

        pub fn create_fungible_and_mint(
            divisibility: u8,
            amount: Decimal,
        ) -> (Bucket, Bucket, ResourceAddress) {
            let badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply(1);
            let token_address = ResourceBuilder::new_fungible()
                .divisibility(divisibility)
                .metadata("name", "TestToken")
                .mintable(rule!(require(badge.resource_address())), LOCKED)
                .burnable(rule!(require(badge.resource_address())), LOCKED)
                .no_initial_supply();
            let tokens = badge.authorize(|| borrow_resource_manager!(token_address).mint(amount));
            (badge, tokens, token_address)
        }

        pub fn create_fungible_wrong_resource_flags_should_fail() -> ResourceAddress {
            let token_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .no_initial_supply();
            token_address
        }

        pub fn create_fungible_wrong_mutable_flags_should_fail() -> ResourceAddress {
            let token_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .no_initial_supply();
            token_address
        }

        pub fn create_fungible_wrong_resource_permissions_should_fail() -> (Bucket, ResourceAddress)
        {
            let badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply(1);
            let token_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .mintable(rule!(require(badge.resource_address())), LOCKED)
                .burnable(rule!(require(badge.resource_address())), LOCKED)
                .no_initial_supply();
            (badge, token_address)
        }

        pub fn query() -> (Bucket, HashMap<String, String>, Decimal) {
            let (badge, resource_address) = Self::create_fungible();
            let resource_manager = borrow_resource_manager!(resource_address);
            (
                badge,
                resource_manager.metadata(),
                resource_manager.total_supply(),
            )
        }

        pub fn burn() -> Bucket {
            let (badge, resource_address) = Self::create_fungible();
            let resource_manager = borrow_resource_manager!(resource_address);
            badge.authorize(|| {
                let bucket: Bucket = resource_manager.mint(1);
                resource_manager.burn(bucket)
            });
            badge
        }

        pub fn update_resource_metadata() -> Bucket {
            let badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply(1);
            let token_resource_manager = borrow_resource_manager!(ResourceBuilder::new_fungible()
                .updateable_metadata(rule!(require(badge.resource_address())), LOCKED)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .no_initial_supply());

            let mut new_metadata = HashMap::new();
            new_metadata.insert("a".to_owned(), "b".to_owned());
            badge.authorize(|| {
                token_resource_manager.update_metadata(new_metadata.clone());
                assert_eq!(token_resource_manager.metadata(), new_metadata);
            });

            badge
        }
    }
}
