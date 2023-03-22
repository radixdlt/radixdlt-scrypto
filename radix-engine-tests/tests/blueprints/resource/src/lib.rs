use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct Sandwich {
    pub name: String,
    #[mutable]
    pub available: bool,
}

#[blueprint]
mod resource_test {
    struct ResourceTest;

    impl ResourceTest {
        pub fn set_mintable_with_self_resource_address() {
            let super_admin_badge: ResourceAddress =
                ResourceBuilder::new_uuid_non_fungible::<Sandwich>()
                    .metadata("name", "Super Admin Badge")
                    .mintable(rule!(allow_all), rule!(allow_all))
                    .create_with_no_initial_supply();

            let super_admin_manager = borrow_resource_manager!(super_admin_badge);
            super_admin_manager.set_mintable(rule!(require(super_admin_badge)));
        }

        pub fn create_fungible() -> (Bucket, ResourceAddress) {
            let badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1);
            let token_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .mintable(rule!(require(badge.resource_address())), rule!(deny_all))
                .burnable(rule!(require(badge.resource_address())), rule!(deny_all))
                .create_with_no_initial_supply();
            (badge, token_address)
        }

        pub fn create_fungible_and_mint(
            divisibility: u8,
            amount: Decimal,
        ) -> (Bucket, Bucket, ResourceAddress) {
            let badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1);
            let token_address = ResourceBuilder::new_fungible()
                .divisibility(divisibility)
                .metadata("name", "TestToken")
                .mintable(rule!(require(badge.resource_address())), rule!(deny_all))
                .burnable(rule!(require(badge.resource_address())), rule!(deny_all))
                .create_with_no_initial_supply();
            let tokens = badge.authorize(|| borrow_resource_manager!(token_address).mint(amount));
            (badge, tokens, token_address)
        }

        pub fn create_fungible_wrong_resource_flags_should_fail() -> Bucket {
            let bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .mint_initial_supply(1u32);
            bucket
        }

        pub fn create_fungible_wrong_mutable_flags_should_fail() -> Bucket {
            let bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .mint_initial_supply(1u32);
            bucket
        }

        pub fn create_fungible_wrong_resource_permissions_should_fail() -> (Bucket, ResourceAddress)
        {
            let badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1);
            let token_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .mintable(rule!(require(badge.resource_address())), rule!(deny_all))
                .burnable(rule!(require(badge.resource_address())), rule!(deny_all))
                .create_with_no_initial_supply();
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
            let badge = ResourceBuilder::new_integer_non_fungible::<Sandwich>()
                .mint_initial_supply(vec![(
                    0u64.into(),
                    Sandwich {
                        name: "name".to_string(),
                        available: false,
                    },
                )]);
            let manager_badge =
                NonFungibleGlobalId::new(badge.resource_address(), NonFungibleLocalId::integer(0));

            let resource_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .owner_non_fungible_badge(manager_badge)
                .create_with_no_initial_supply();

            badge.authorize(|| {
                let token_resource_manager = borrow_resource_manager!(resource_address);
                let metadata = token_resource_manager.metadata();
                metadata.set("a".to_owned(), "b".to_owned());
                assert_eq!(metadata.get_string("a".to_owned()).unwrap(), "b".to_owned());
            });

            badge
        }
    }
}

#[blueprint]
mod auth_resource {
    struct AuthResource;

    impl AuthResource {
        pub fn create() -> ComponentAddress {
            Self {}.instantiate().globalize()
        }

        pub fn mint(&self, resource: ResourceAddress) -> Bucket {
            let resource_manager = borrow_resource_manager!(resource);
            let bucket = resource_manager.mint(1);
            bucket
        }

        pub fn burn(&self, bucket: Bucket) {
            bucket.burn();
        }
    }
}
