use scrypto::prelude::*;

blueprint! {
    struct ResourceTest;

    impl ResourceTest {
        pub fn create_fungible() -> (Bucket, ResourceDefId) {
            let badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE).initial_supply_fungible(1);
            let token_resource_def = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .flags(MINTABLE | BURNABLE)
                .badge(badge.resource_def_id(), MAY_MINT | MAY_BURN)
                .no_initial_supply();
            (badge, token_resource_def)
        }

        pub fn create_fungible_should_fail() -> (Bucket, Bucket) {
            let mut bucket =
                ResourceBuilder::new_fungible(DIVISIBILITY_NONE).initial_supply_fungible(1);
            (bucket.take(Decimal::from_str("0.1").unwrap()), bucket)
        }

        pub fn create_fungible_wrong_resource_flags_should_fail() -> ResourceDefId {
            let token_resource_def = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .flags(MAY_MINT | BURNABLE)
                .no_initial_supply();
            token_resource_def
        }

        pub fn create_fungible_wrong_mutable_flags_should_fail() -> ResourceDefId {
            let token_resource_def = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .flags(MINTABLE | BURNABLE)
                .mutable_flags(MAY_MINT)
                .no_initial_supply();
            token_resource_def
        }

        pub fn create_fungible_wrong_resource_permissions_should_fail() -> (Bucket, ResourceDefId) {
            let badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE).initial_supply_fungible(1);
            let token_resource_def = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .flags(MINTABLE | BURNABLE)
                .badge(badge.resource_def_id(), MINTABLE | MAY_BURN)
                .no_initial_supply();
            (badge, token_resource_def)
        }

        pub fn query() -> (Bucket, HashMap<String, String>, u64, u64, Decimal) {
            let (badge, resource_def_id) = Self::create_fungible();
            let resource_def = resource_def!(resource_def_id);
            (
                badge,
                resource_def.metadata(),
                resource_def.flags(),
                resource_def.mutable_flags(),
                resource_def.total_supply(),
            )
        }

        pub fn burn() -> Bucket {
            let (badge, resource_def_id) = Self::create_fungible();
            let resource_def = resource_def!(resource_def_id);
            let bucket = authorize(&badge, || resource_def.mint(1));
            resource_def.burn_with_auth(bucket, badge.present());
            badge
        }

        pub fn update_feature_flags() -> Bucket {
            let badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE).initial_supply_fungible(1);
            let token_resource_def =
                resource_def!(ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                    .metadata("name", "TestToken")
                    .mutable_flags(MINTABLE)
                    .badge(
                        badge.resource_def_id(),
                        MAY_MANAGE_RESOURCE_FLAGS | MAY_MINT,
                    )
                    .no_initial_supply());

            token_resource_def.enable_flags(MINTABLE, badge.present());
            assert!(token_resource_def.flags() & MINTABLE == MINTABLE);
            assert!(token_resource_def.mutable_flags() & MINTABLE == MINTABLE);

            token_resource_def.disable_flags(MINTABLE, badge.present());
            assert!(token_resource_def.flags() & MINTABLE == 0);
            assert!(token_resource_def.mutable_flags() & MINTABLE == MINTABLE);

            token_resource_def.lock_flags(MINTABLE, badge.present());
            assert!(token_resource_def.flags() & MINTABLE == 0);
            assert!(token_resource_def.mutable_flags() & MINTABLE == 0);

            badge
        }

        pub fn update_feature_flags_should_fail() -> Bucket {
            let badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE).initial_supply_fungible(1);
            let token_resource_def =
                resource_def!(ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                    .metadata("name", "TestToken")
                    .badge(
                        badge.resource_def_id(),
                        MAY_MANAGE_RESOURCE_FLAGS | MAY_MINT,
                    )
                    .no_initial_supply());

            token_resource_def.enable_flags(MINTABLE, badge.present());
            badge
        }

        pub fn update_resource_metadata() -> Bucket {
            let badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE).initial_supply_fungible(1);
            let token_resource_def =
                resource_def!(ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                    .metadata("name", "TestToken")
                    .flags(SHARED_METADATA_MUTABLE)
                    .badge(badge.resource_def_id(), MAY_CHANGE_SHARED_METADATA)
                    .no_initial_supply());

            let mut new_metadata = HashMap::new();
            new_metadata.insert("a".to_owned(), "b".to_owned());
            token_resource_def.update_metadata(new_metadata.clone(), badge.present());
            assert_eq!(token_resource_def.metadata(), new_metadata);

            badge
        }
    }
}
