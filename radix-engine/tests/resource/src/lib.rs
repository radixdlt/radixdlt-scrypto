use scrypto::prelude::*;

blueprint! {
    struct ResourceTest;

    impl ResourceTest {
        pub fn create_fungible() -> (Bucket, ResourceDefId) {
            let badge = ResourceBuilder::new_fungible()
                .auth("take_from_vault", auth!(allow_all))
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply(1);
            let token_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .auth("take_from_vault", auth!(allow_all))
                .auth("mint", auth!(require(badge.resource_def_id())))
                .auth("burn", auth!(require(badge.resource_def_id())))
                .no_initial_supply();
            (badge, token_address)
        }

        pub fn create_fungible_and_mint(
            divisibility: u8,
            amount: Decimal,
        ) -> (Bucket, Bucket, ResourceDefId) {
            let badge = ResourceBuilder::new_fungible()
                .auth("take_from_vault", auth!(allow_all))
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply(1);
            let token_address = ResourceBuilder::new_fungible()
                .divisibility(divisibility)
                .metadata("name", "TestToken")
                .auth("take_from_vault", auth!(allow_all))
                .auth("mint", auth!(require(badge.resource_def_id())))
                .auth("burn", auth!(require(badge.resource_def_id())))
                .no_initial_supply();
            let tokens = badge.authorize(|| resource_def!(token_address).mint(amount));
            (badge, tokens, token_address)
        }

        pub fn create_fungible_wrong_resource_flags_should_fail() -> ResourceDefId {
            let token_address = ResourceBuilder::new_fungible()
                .auth("take_from_vault", auth!(allow_all))
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .no_initial_supply();
            token_address
        }

        pub fn create_fungible_wrong_mutable_flags_should_fail() -> ResourceDefId {
            let token_address = ResourceBuilder::new_fungible()
                .auth("take_from_vault", auth!(allow_all))
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .flags(MINTABLE | BURNABLE)
                .no_initial_supply();
            token_address
        }

        pub fn create_fungible_wrong_resource_permissions_should_fail() -> (Bucket, ResourceDefId) {
            let badge = ResourceBuilder::new_fungible()
                .auth("take_from_vault", auth!(allow_all))
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply(1);
            let token_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .flags(MINTABLE | BURNABLE)
                .auth("take_from_vault", auth!(allow_all))
                .auth("mint", auth!(require(badge.resource_def_id())))
                .auth("burn", auth!(require(badge.resource_def_id())))
                .no_initial_supply();
            (badge, token_address)
        }

        pub fn query() -> (Bucket, HashMap<String, String>, u64, Decimal) {
            let (badge, resource_def_id) = Self::create_fungible();
            let resource_def = resource_def!(resource_def_id);
            (
                badge,
                resource_def.metadata(),
                resource_def.flags(),
                resource_def.total_supply(),
            )
        }

        pub fn burn() -> Bucket {
            let (badge, resource_def_id) = Self::create_fungible();
            let resource_def = resource_def!(resource_def_id);
            badge.authorize(|| {
                let bucket: Bucket = resource_def.mint(1);
                resource_def.burn(bucket)
            });
            badge
        }

        pub fn update_resource_metadata() -> Bucket {
            let badge = ResourceBuilder::new_fungible()
                .auth("take_from_vault", auth!(allow_all))
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply(1);
            let token_resource_def = resource_def!(ResourceBuilder::new_fungible()
                .auth("take_from_vault", auth!(allow_all))
                .auth("update_metadata", auth!(require(badge.resource_def_id())))
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .flags(SHARED_METADATA_MUTABLE)
                .no_initial_supply());

            let mut new_metadata = HashMap::new();
            new_metadata.insert("a".to_owned(), "b".to_owned());
            badge.authorize(|| {
                token_resource_def.update_metadata(new_metadata.clone());
                assert_eq!(token_resource_def.metadata(), new_metadata);
            });

            badge
        }
    }
}

package_init!(blueprint::ResourceTest::describe());
