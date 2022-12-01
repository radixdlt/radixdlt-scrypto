use scrypto::prelude::*;

blueprint! {
    struct ResourceTest;

    impl ResourceTest {
        pub fn set_mintable_with_self_resource_address() {
            let super_admin_badge: ResourceAddress = ResourceBuilder::new_non_fungible()
                .metadata("name", "Super Admin Badge")
                .mintable(rule!(allow_all), MUTABLE(rule!(allow_all)))
                .no_initial_supply_no_owner();

            let super_admin_manager: &mut ResourceManager =
                borrow_resource_manager!(super_admin_badge);
            super_admin_manager.set_mintable(rule!(require(super_admin_badge)));
        }

        pub fn create_fungible() -> (Bucket, ResourceAddress) {
            let badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply_no_owner(1);
            let token_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .mintable(rule!(require(badge.resource_address())), LOCKED)
                .burnable(rule!(require(badge.resource_address())), LOCKED)
                .no_initial_supply_no_owner();
            (badge, token_address)
        }

        pub fn create_fungible_and_mint(
            divisibility: u8,
            amount: Decimal,
        ) -> (Bucket, Bucket, ResourceAddress) {
            let badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply_no_owner(1);
            let token_address = ResourceBuilder::new_fungible()
                .divisibility(divisibility)
                .metadata("name", "TestToken")
                .mintable(rule!(require(badge.resource_address())), LOCKED)
                .burnable(rule!(require(badge.resource_address())), LOCKED)
                .no_initial_supply_no_owner();
            let tokens = badge.authorize(|| borrow_resource_manager!(token_address).mint(amount));
            (badge, tokens, token_address)
        }

        pub fn create_fungible_wrong_resource_flags_should_fail() -> ResourceAddress {
            let token_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .no_initial_supply_no_owner();
            token_address
        }

        pub fn create_fungible_wrong_mutable_flags_should_fail() -> ResourceAddress {
            let token_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .no_initial_supply_no_owner();
            token_address
        }

        pub fn create_fungible_wrong_resource_permissions_should_fail() -> (Bucket, ResourceAddress)
        {
            let badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply_no_owner(1);
            let token_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .mintable(rule!(require(badge.resource_address())), LOCKED)
                .burnable(rule!(require(badge.resource_address())), LOCKED)
                .no_initial_supply_no_owner();
            (badge, token_address)
        }

        pub fn query() -> (Bucket, Decimal, ResourceType) {
            let (badge, resource_address) = Self::create_fungible();
            let resource_manager = borrow_resource_manager!(resource_address);
            (
                badge,
                resource_manager.total_supply(),
                resource_manager.resource_type(),
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
            let (resource_address, badge) = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .no_initial_supply_with_owner();
            let token_resource_manager = borrow_resource_manager!(resource_address);

            badge.authorize(|| {
                token_resource_manager.set_metadata("a".to_owned(), "b".to_owned());
                assert_eq!(
                    token_resource_manager.get_metadata("a".to_owned()).unwrap(),
                    "b".to_owned()
                );
            });

            badge
        }
    }
}
