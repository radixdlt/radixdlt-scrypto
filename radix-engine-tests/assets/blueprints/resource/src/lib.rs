use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct TestNFData {
    pub name: String,
    #[mutable]
    pub available: bool,
}

#[blueprint]
mod resource_test {
    struct ResourceTest {
        vault: NonFungibleVault,
        data: String,
    }

    impl ResourceTest {
        pub fn non_fungible_global_id() {
            let bucket = ResourceBuilder::new_integer_non_fungible::<TestNFData>(OwnerRole::None)
                .mint_initial_supply(vec![(
                    0u64.into(),
                    TestNFData {
                        name: "name".to_string(),
                        available: false,
                    },
                )]);
            let expected =
                NonFungibleGlobalId::new(bucket.resource_address(), NonFungibleLocalId::integer(0));
            let nf_global_id1 = bucket.non_fungible_global_id();

            let proof = bucket
                .create_proof_of_non_fungibles(&indexset!(NonFungibleLocalId::integer(0)))
                .skip_checking();
            let nf_global_id2 = proof.non_fungible_global_id();
            proof.drop();

            let vault = NonFungibleVault::with_bucket(bucket);
            let nf_global_id3 = vault.non_fungible_global_id();

            Self {
                vault: vault.into(),
                data: "hi".to_owned(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();

            assert_eq!(nf_global_id1, expected);
            assert_eq!(nf_global_id2, expected);
            assert_eq!(nf_global_id3, expected);
        }

        pub fn take_from_vault_after_mint() {
            let bucket = ResourceBuilder::new_integer_non_fungible::<TestNFData>(OwnerRole::None)
                .mint_initial_supply(vec![(
                    0u64.into(),
                    TestNFData {
                        name: "name".to_string(),
                        available: false,
                    },
                )]);
            let global = Self {
                vault: NonFungibleVault::new(bucket.resource_address()),
                data: "hi".to_string(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();

            global.take_from_vault_after_mint_helper(bucket);
        }

        pub fn take_from_vault_after_mint_helper(&mut self, bucket: NonFungibleBucket) {
            self.vault.put(bucket);
            let bucket = self.vault.take(dec!(1));
            self.vault.put(bucket);
            self.data = "hello".to_string();
        }

        pub fn query_nonexistent_and_mint() {
            let resource_manager =
                ResourceBuilder::new_integer_non_fungible::<TestNFData>(OwnerRole::None)
                    .mint_roles(mint_roles! {
                        minter => rule!(allow_all);
                        minter_updater => rule!(allow_all);
                    })
                    .burn_roles(burn_roles! {
                        burner => rule!(allow_all);
                        burner_updater => rule!(allow_all);
                    })
                    .create_with_no_initial_supply();

            let global = Self {
                vault: NonFungibleVault::new(resource_manager.address()),
                data: "hi".to_string(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();

            global.query_nonexistent_and_mint_helper();
        }

        pub fn query_nonexistent_and_mint_helper(&mut self) {
            self.vault
                .contains_non_fungible(&NonFungibleLocalId::integer(0));
            let bucket = self.vault.resource_manager().mint_non_fungible(
                &NonFungibleLocalId::integer(0),
                TestNFData {
                    name: "name".to_string(),
                    available: false,
                },
            );
            self.vault.put(bucket);
        }

        pub fn set_mintable_with_self_resource_address() {
            let super_admin_manager =
                ResourceBuilder::new_ruid_non_fungible::<TestNFData>(OwnerRole::None)
                    .metadata(metadata! {
                        init {
                            "name" => "Super Admin Badge".to_owned(), locked;
                        }
                    })
                    .mint_roles(mint_roles! {
                        minter => rule!(allow_all);
                        minter_updater => rule!(allow_all);
                    })
                    .create_with_no_initial_supply();

            super_admin_manager.set_mintable(rule!(require(super_admin_manager.address())));
        }

        pub fn create_fungible() -> (Bucket, FungibleResourceManager) {
            let badge = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1);
            let resource_manager = ResourceBuilder::new_fungible(OwnerRole::Fixed(rule!(require(
                badge.resource_address()
            ))))
            .divisibility(DIVISIBILITY_MAXIMUM)
            .metadata(metadata! {
                init {
                    "name" => "TestToken".to_owned(), locked;
                }
            })
            .mint_roles(mint_roles! {
                minter => OWNER;
                minter_updater => rule!(deny_all);
            })
            .burn_roles(burn_roles! {
                burner => OWNER;
                burner_updater => rule!(deny_all);
            })
            .create_with_no_initial_supply();
            (badge.into(), resource_manager)
        }

        pub fn create_fungible_and_mint(
            divisibility: u8,
            amount: Decimal,
        ) -> (Bucket, Bucket, FungibleResourceManager) {
            let badge = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1);
            let resource_manager = ResourceBuilder::new_fungible(OwnerRole::Fixed(rule!(require(
                badge.resource_address()
            ))))
            .divisibility(divisibility)
            .metadata(metadata! {
                init {
                    "name" => "TestToken".to_owned(), locked;
                }
            })
            .mint_roles(mint_roles! {
                minter => OWNER;
                minter_updater => rule!(deny_all);
            })
            .burn_roles(burn_roles! {
                burner => OWNER;
                burner_updater => rule!(deny_all);
            })
            .create_with_no_initial_supply();
            let tokens = badge.authorize_with_amount(dec!(1), || resource_manager.mint(amount));
            (badge.into(), tokens.into(), resource_manager)
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
            bucket.into()
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
            bucket.into()
        }

        pub fn create_fungible_wrong_resource_permissions_should_fail(
        ) -> (Bucket, FungibleResourceManager) {
            let badge = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1);
            let resource_manager = ResourceBuilder::new_fungible(OwnerRole::Fixed(rule!(require(
                badge.resource_address()
            ))))
            .divisibility(DIVISIBILITY_MAXIMUM)
            .metadata(metadata! {
                init {
                    "name" => "TestToken".to_owned(), locked;
                }
            })
            .mint_roles(mint_roles! {
                minter => OWNER;
                minter_updater => rule!(deny_all);
            })
            .burn_roles(burn_roles! {
                burner => OWNER;
                burner_updater => rule!(deny_all);
            })
            .create_with_no_initial_supply();
            (badge.into(), resource_manager)
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
            badge.as_fungible().authorize_with_amount(dec!(1), || {
                let bucket = resource_manager.mint(1);
                resource_manager.burn(bucket)
            });
            badge
        }

        pub fn update_resource_metadata() -> Bucket {
            let badge: Bucket =
                ResourceBuilder::new_integer_non_fungible::<TestNFData>(OwnerRole::None)
                    .mint_initial_supply(vec![(
                        0u64.into(),
                        TestNFData {
                            name: "name".to_string(),
                            available: false,
                        },
                    )])
                    .into();
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

            badge.authorize_with_all(|| {
                token_resource_manager.set_metadata("a".to_owned(), "b".to_owned());
                let string: String = token_resource_manager
                    .get_metadata("a".to_owned())
                    .unwrap()
                    .unwrap();
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

        pub fn mint(&self, resource_manager: FungibleResourceManager) -> FungibleBucket {
            let bucket = resource_manager.mint(1);
            bucket
        }

        pub fn burn(&self, bucket: FungibleBucket) {
            bucket.burn();
        }
    }
}

#[blueprint]
mod rounding {
    struct RoundingTest {
        vault: Vault,
    }

    impl RoundingTest {
        pub fn fungible_resource_amount_for_withdrawal() -> Bucket {
            let bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(2)
                .mint_initial_supply(100);
            let manager = bucket.resource_manager();
            assert_eq!(
                manager.amount_for_withdrawal(dec!("1.515"), WithdrawStrategy::Exact),
                dec!("1.515")
            );
            assert_eq!(
                manager.amount_for_withdrawal(
                    dec!("1.515"),
                    WithdrawStrategy::Rounded(RoundingMode::ToZero)
                ),
                dec!("1.51")
            );
            assert_eq!(
                manager.amount_for_withdrawal(
                    dec!("1.515"),
                    WithdrawStrategy::Rounded(RoundingMode::ToNearestMidpointToEven)
                ),
                dec!("1.52")
            );
            bucket.into()
        }

        pub fn non_fungible_resource_amount_for_withdrawal() -> Bucket {
            let bucket: Bucket =
                ResourceBuilder::new_integer_non_fungible::<TestNFData>(OwnerRole::None)
                    .mint_initial_supply(vec![(
                        0u64.into(),
                        TestNFData {
                            name: "name".to_string(),
                            available: false,
                        },
                    )])
                    .into();
            let manager = bucket.resource_manager();
            assert_eq!(
                manager.amount_for_withdrawal(dec!("1.515"), WithdrawStrategy::Exact),
                dec!("1.515")
            );
            assert_eq!(
                manager.amount_for_withdrawal(
                    dec!("1.515"),
                    WithdrawStrategy::Rounded(RoundingMode::ToZero)
                ),
                dec!(1)
            );
            assert_eq!(
                manager.amount_for_withdrawal(
                    dec!("1.515"),
                    WithdrawStrategy::Rounded(RoundingMode::ToNearestMidpointToEven)
                ),
                dec!("2")
            );
            bucket
        }

        pub fn fungible_resource_take_advanced() {
            let mut bucket = Self::fungible_resource_amount_for_withdrawal();
            let bucket2 = bucket.take_advanced(
                dec!("1.231"),
                WithdrawStrategy::Rounded(RoundingMode::ToZero),
            );
            assert_eq!(bucket2.amount(), dec!("1.23"));
            bucket.put(bucket2);

            let mut vault = Vault::with_bucket(bucket);
            let bucket2 = vault.take_advanced(
                dec!("1.231"),
                WithdrawStrategy::Rounded(RoundingMode::ToZero),
            );
            assert_eq!(bucket2.amount(), dec!("1.23"));
            vault.put(bucket2);

            Self { vault }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize();
        }

        pub fn fungible_bucket_take_advanced_max() {
            let mut bucket = Self::fungible_resource_amount_for_withdrawal();
            let _ = bucket.take_advanced(
                Decimal::MAX,
                WithdrawStrategy::Rounded(RoundingMode::AwayFromZero),
            );
        }

        pub fn fungible_vault_take_advanced_max() {
            let bucket = Self::fungible_resource_amount_for_withdrawal();
            let mut vault = Vault::with_bucket(bucket);
            let _ = vault.take_advanced(
                Decimal::MAX,
                WithdrawStrategy::Rounded(RoundingMode::AwayFromZero),
            );
        }

        pub fn non_fungible_bucket_take_advanced_max() {
            let mut bucket = Self::non_fungible_resource_amount_for_withdrawal();
            let _ = bucket.take_advanced(
                Decimal::MAX,
                WithdrawStrategy::Rounded(RoundingMode::AwayFromZero),
            );
        }

        pub fn non_fungible_vault_take_advanced_max() {
            let bucket = Self::non_fungible_resource_amount_for_withdrawal();
            let mut vault = Vault::with_bucket(bucket);
            let _ = vault.take_advanced(
                Decimal::MAX,
                WithdrawStrategy::Rounded(RoundingMode::AwayFromZero),
            );
        }

        pub fn non_fungible_resource_take_advanced() {
            let mut bucket = Self::non_fungible_resource_amount_for_withdrawal();
            let bucket2 = bucket.take_advanced(
                dec!("1.231"),
                WithdrawStrategy::Rounded(RoundingMode::ToZero),
            );
            assert_eq!(bucket2.amount(), dec!(1));
            bucket.put(bucket2);

            let mut vault = Vault::with_bucket(bucket);
            let bucket2 = vault.take_advanced(
                dec!("1.231"),
                WithdrawStrategy::Rounded(RoundingMode::ToZero),
            );
            assert_eq!(bucket2.amount(), dec!(1));
            vault.put(bucket2);

            Self { vault }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize();
        }
    }
}

#[blueprint]
mod resource_types {
    struct ResourceTypes {
        fungible_vault: Option<FungibleVault>,
        non_fungible_vault: Option<NonFungibleVault>,
    }

    impl ResourceTypes {
        pub fn test_fungible_types() {
            let x: (FungibleBucket, FungibleProof, FungibleVault) =
                Blueprint::<ResourceTypes>::call_function("produce_fungible_things", ());
            let _: () = Blueprint::<ResourceTypes>::call_function("consume_fungible_things", x);
        }

        pub fn test_non_fungible_types() {
            let x: (NonFungibleBucket, NonFungibleProof, NonFungibleVault) =
                Blueprint::<ResourceTypes>::call_function("produce_non_fungible_things", ());
            let _: () = Blueprint::<ResourceTypes>::call_function("consume_non_fungible_things", x);
        }

        pub fn produce_fungible_things() -> (FungibleBucket, FungibleProof, FungibleVault) {
            let mut bucket =
                ResourceBuilder::new_fungible(OwnerRole::None).mint_initial_supply(100);
            let proof = bucket.create_proof_of_amount(dec!(1));
            let vault = FungibleVault::with_bucket(bucket.take(5));

            (bucket, proof, vault)
        }

        pub fn consume_fungible_things(
            bucket: FungibleBucket,
            proof: FungibleProof,
            mut vault: FungibleVault,
        ) {
            proof.drop();
            vault.put(bucket);
            Self {
                fungible_vault: vault.into(),
                non_fungible_vault: None,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();
        }

        pub fn produce_non_fungible_things(
        ) -> (NonFungibleBucket, NonFungibleProof, NonFungibleVault) {
            let mut bucket =
                ResourceBuilder::new_integer_non_fungible::<TestNFData>(OwnerRole::None)
                    .mint_initial_supply(vec![
                        (
                            0u64.into(),
                            TestNFData {
                                name: "A".to_string(),
                                available: true,
                            },
                        ),
                        (
                            1u64.into(),
                            TestNFData {
                                name: "B".to_string(),
                                available: true,
                            },
                        ),
                    ]);
            let proof =
                bucket.create_proof_of_non_fungibles(&indexset!(NonFungibleLocalId::integer(0)));
            let vault = NonFungibleVault::with_bucket(bucket.take(1));

            (bucket, proof, vault)
        }

        pub fn consume_non_fungible_things(
            bucket: NonFungibleBucket,
            proof: NonFungibleProof,
            mut vault: NonFungibleVault,
        ) {
            proof.drop();
            vault.put(bucket);
            Self {
                non_fungible_vault: vault.into(),
                fungible_vault: None,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();
        }
    }
}
