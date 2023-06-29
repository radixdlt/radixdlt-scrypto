use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct TestNFData {
    pub name: String,
    #[mutable]
    pub available: bool,
}

#[blueprint]
mod resource_test {
    struct ResourceTest;

    impl ResourceTest {
        pub fn set_mintable_with_self_resource_address() {
            let super_admin_manager: ResourceManager =
                ResourceBuilder::new_ruid_non_fungible::<TestNFData>(OwnerRole::None)
                    .metadata(metadata! {
                        init {
                            "name" => "Super Admin Badge".to_owned(), locked;
                        }
                    })
                    .mintable(rule!(allow_all), rule!(allow_all))
                    .create_with_no_initial_supply();

            super_admin_manager.set_mintable(rule!(require(super_admin_manager.address())));
        }

        pub fn create_fungible() -> (Bucket, ResourceManager) {
            let badge = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1);
            let resource_manager = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata(metadata! {
                    init {
                        "name" => "TestToken".to_owned(), locked;
                    }
                })
                .mintable(rule!(require(badge.resource_address())), rule!(deny_all))
                .burnable(rule!(require(badge.resource_address())), rule!(deny_all))
                .create_with_no_initial_supply();
            (badge, resource_manager)
        }

        pub fn create_fungible_and_mint(
            divisibility: u8,
            amount: Decimal,
        ) -> (Bucket, Bucket, ResourceManager) {
            let badge = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1);
            let resource_manager = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(divisibility)
                .metadata(metadata! {
                    init {
                        "name" => "TestToken".to_owned(), locked;
                    }
                })
                .mintable(rule!(require(badge.resource_address())), rule!(deny_all))
                .burnable(rule!(require(badge.resource_address())), rule!(deny_all))
                .create_with_no_initial_supply();
            let tokens = badge.authorize(|| resource_manager.mint(amount));
            (badge, tokens, resource_manager)
        }

        pub fn create_fungible_wrong_resource_flags_should_fail() -> Bucket {
            let bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata(metadata! {
                    init {
                        "name" => "TestToken".to_owned(), locked;
                    }
                })
                .mint_initial_supply(1u32);
            bucket
        }

        pub fn create_fungible_wrong_mutable_flags_should_fail() -> Bucket {
            let bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata(metadata! {
                    init {
                        "name" => "TestToken".to_owned(), locked;
                    }
                })
                .mint_initial_supply(1u32);
            bucket
        }

        pub fn create_fungible_wrong_resource_permissions_should_fail() -> (Bucket, ResourceManager)
        {
            let badge = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1);
            let resource_manager = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata(metadata! {
                    init {
                        "name" => "TestToken".to_owned(), locked;
                    }
                })
                .mintable(rule!(require(badge.resource_address())), rule!(deny_all))
                .burnable(rule!(require(badge.resource_address())), rule!(deny_all))
                .create_with_no_initial_supply();
            (badge, resource_manager)
        }

        pub fn query() -> (Bucket, Decimal, ResourceType) {
            let (badge, resource_manager) = Self::create_fungible();
            (
                badge,
                resource_manager.total_supply().unwrap(),
                resource_manager.resource_type(),
            )
        }

        pub fn burn() -> Bucket {
            let (badge, resource_manager) = Self::create_fungible();
            badge.authorize(|| {
                let bucket: Bucket = resource_manager.mint(1);
                resource_manager.burn(bucket)
            });
            badge
        }

        pub fn update_resource_metadata() -> Bucket {
            let badge = ResourceBuilder::new_integer_non_fungible::<TestNFData>(OwnerRole::None)
                .mint_initial_supply(vec![(
                    0u64.into(),
                    TestNFData {
                        name: "name".to_string(),
                        available: false,
                    },
                )]);
            let manager_badge =
                NonFungibleGlobalId::new(badge.resource_address(), NonFungibleLocalId::integer(0));

            let token_resource_manager =
                ResourceBuilder::new_fungible(OwnerRole::Fixed(rule!(require(manager_badge))))
                    .divisibility(DIVISIBILITY_MAXIMUM)
                    .metadata(metadata! {
                        init {
                            "name" => "TestToken".to_owned(), locked;
                        }
                    })
                    .create_with_no_initial_supply();

            badge.authorize(|| {
                token_resource_manager.set_metadata("a".to_owned(), "b".to_owned());
                let string: String = token_resource_manager.get_metadata("a".to_owned()).unwrap();
                assert_eq!(string, "b".to_owned());
            });

            badge
        }
    }
}

#[blueprint]
mod auth_resource {
    struct AuthResource;

    impl AuthResource {
        pub fn create() -> Global<AuthResource> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn mint(&self, resource_manager: ResourceManager) -> Bucket {
            let bucket = resource_manager.mint(1);
            bucket
        }

        pub fn burn(&self, bucket: Bucket) {
            bucket.burn();
        }
    }
}
