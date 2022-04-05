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
                .flags(MINTABLE | BURNABLE)
                .badge(badge.resource_address(), MAY_MINT | MAY_BURN)
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
                .flags(MINTABLE | BURNABLE)
                .badge(badge.resource_address(), MAY_MINT | MAY_BURN)
                .no_initial_supply();
            let tokens = badge.authorize(|| resource_manager!(token_address).mint(amount));
            (badge, tokens, token_address)
        }

        pub fn create_fungible_wrong_resource_flags_should_fail() -> ResourceAddress {
            let token_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .flags(MAY_MINT | BURNABLE)
                .no_initial_supply();
            token_address
        }

        pub fn create_fungible_wrong_mutable_flags_should_fail() -> ResourceAddress {
            let token_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .flags(MINTABLE | BURNABLE)
                .mutable_flags(MAY_MINT)
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
                .flags(MINTABLE | BURNABLE)
                .badge(badge.resource_address(), MINTABLE | MAY_BURN)
                .no_initial_supply();
            (badge, token_address)
        }

        pub fn query() -> (Bucket, HashMap<String, String>, u64, u64, Decimal) {
            let (badge, resource_address) = Self::create_fungible();
            let resource_manager = resource_manager!(resource_address);
            (
                badge,
                resource_manager.metadata(),
                resource_manager.flags(),
                resource_manager.mutable_flags(),
                resource_manager.total_supply(),
            )
        }

        pub fn burn() -> Bucket {
            let (badge, resource_address) = Self::create_fungible();
            let resource_manager = resource_manager!(resource_address);
            badge.authorize(|| {
                let bucket: Bucket = resource_manager.mint(1);
                resource_manager.burn(bucket)
            });
            badge
        }

        pub fn update_feature_flags() -> Bucket {
            let badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply(1);
            let token_resource_manager = resource_manager!(ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .mutable_flags(MINTABLE)
                .badge(
                    badge.resource_address(),
                    MAY_MANAGE_RESOURCE_FLAGS | MAY_MINT,
                )
                .no_initial_supply());

            badge.authorize(|| {
                token_resource_manager.enable_flags(MINTABLE);
                assert!(token_resource_manager.flags() & MINTABLE == MINTABLE);
                assert!(token_resource_manager.mutable_flags() & MINTABLE == MINTABLE);

                token_resource_manager.disable_flags(MINTABLE);
                assert!(token_resource_manager.flags() & MINTABLE == 0);
                assert!(token_resource_manager.mutable_flags() & MINTABLE == MINTABLE);

                token_resource_manager.lock_flags(MINTABLE);
                assert!(token_resource_manager.flags() & MINTABLE == 0);
                assert!(token_resource_manager.mutable_flags() & MINTABLE == 0);
            });

            badge
        }

        pub fn update_feature_flags_should_fail() -> Bucket {
            let badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply(1);
            let token_resource_manager = resource_manager!(ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .badge(
                    badge.resource_address(),
                    MAY_MANAGE_RESOURCE_FLAGS | MAY_MINT,
                )
                .no_initial_supply());

            badge.authorize(|| token_resource_manager.enable_flags(MINTABLE));
            badge
        }

        pub fn update_resource_metadata() -> Bucket {
            let badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply(1);
            let token_resource_manager = resource_manager!(ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .flags(SHARED_METADATA_MUTABLE)
                .badge(badge.resource_address(), MAY_CHANGE_SHARED_METADATA)
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
