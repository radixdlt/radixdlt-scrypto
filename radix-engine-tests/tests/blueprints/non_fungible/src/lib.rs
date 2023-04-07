use scrypto::api::ClientObjectApi;
use scrypto::engine::scrypto_env::ScryptoEnv;
use scrypto::prelude::*;

#[derive(Debug, PartialEq, Eq, ScryptoSbor, NonFungibleData)]
pub struct Sandwich {
    pub name: String,
    #[mutable]
    pub available: bool,
    pub tastes_great: bool,
    #[mutable]
    pub reference: Option<ComponentAddress>,
    #[mutable]
    pub own: Option<Own>,
}

#[blueprint]
mod non_fungible_test {
    struct NonFungibleTest {
        vault: Vault,
    }

    impl NonFungibleTest {
        pub fn create_non_fungible_mutable() -> (Bucket, ResourceAddress, Bucket) {
            // Create a mint badge
            let mint_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1);

            // Create non-fungible resource with mutable supply
            let resource_address = ResourceBuilder::new_integer_non_fungible::<Sandwich>()
                .metadata("name", "Katz's Sandwiches")
                .mintable(
                    rule!(require(mint_badge.resource_address())),
                    rule!(deny_all),
                )
                .burnable(rule!(allow_all), rule!(deny_all))
                .updateable_non_fungible_data(
                    rule!(require(mint_badge.resource_address())),
                    rule!(deny_all),
                )
                .create_with_no_initial_supply();

            // Mint a non-fungible
            let non_fungible = mint_badge.authorize(|| {
                borrow_resource_manager!(resource_address).mint_non_fungible(
                    &NonFungibleLocalId::integer(0),
                    Sandwich {
                        name: "Test".to_owned(),
                        available: false,
                        tastes_great: true,
                        reference: None,
                        own: None,
                    },
                )
            });

            (mint_badge, resource_address, non_fungible)
        }

        pub fn update_nft(mint_badge: Bucket, proof: Proof) -> Bucket {
            let proof = proof.unsafe_skip_proof_validation();
            mint_badge.authorize(|| {
                borrow_resource_manager!(proof.resource_address()).update_non_fungible_data(
                    &proof.non_fungible_local_id(),
                    "available",
                    true,
                )
            });

            mint_badge
        }

        pub fn create_burnable_non_fungible() -> Bucket {
            ResourceBuilder::new_uuid_non_fungible()
                .metadata("name", "Katz's Sandwiches")
                .burnable(rule!(allow_all), rule!(deny_all))
                .mint_initial_supply([
                    Sandwich {
                        name: "Zero".to_owned(),
                        available: true,
                        tastes_great: true,
                        reference: None,
                        own: None,
                    },
                    Sandwich {
                        name: "One".to_owned(),
                        available: true,
                        tastes_great: true,
                        reference: None,
                        own: None,
                    },
                ])
        }

        pub fn create_non_fungible_fixed() -> Bucket {
            ResourceBuilder::new_integer_non_fungible()
                .metadata("name", "Katz's Sandwiches")
                .mint_initial_supply([
                    (
                        1u64.into(),
                        Sandwich {
                            name: "One".to_owned(),
                            available: true,
                            tastes_great: true,
                            reference: None,
                            own: None,
                        },
                    ),
                    (
                        2u64.into(),
                        Sandwich {
                            name: "Two".to_owned(),
                            available: true,
                            tastes_great: true,
                            reference: None,
                            own: None,
                        },
                    ),
                    (
                        3u64.into(),
                        Sandwich {
                            name: "Three".to_owned(),
                            available: true,
                            tastes_great: true,
                            reference: None,
                            own: None,
                        },
                    ),
                ])
        }

        pub fn verify_does_not_exist(non_fungible_global_id: NonFungibleGlobalId) {
            assert_eq!(
                borrow_resource_manager!(non_fungible_global_id.resource_address())
                    .non_fungible_exists(&non_fungible_global_id.local_id()),
                false
            );
        }

        pub fn create_non_fungible_reference(address: ComponentAddress) -> (Bucket, Bucket) {
            let (mint_badge, resource_address, bucket) = Self::create_non_fungible_mutable();

            mint_badge.authorize(|| {
                borrow_resource_manager!(resource_address).update_non_fungible_data(
                    &NonFungibleLocalId::integer(0),
                    "reference",
                    Some(address),
                );
            });

            (mint_badge, bucket)
        }

        pub fn call_non_fungible_reference(resource_address: ResourceAddress) -> String {
            let data: Sandwich = borrow_resource_manager!(resource_address)
                .get_non_fungible_data(&NonFungibleLocalId::integer(0));
            let address = data.reference.unwrap();

            let metadata = borrow_component!(address).metadata();
            metadata.get_string("test_key").unwrap()
        }

        pub fn update_non_fungible_with_ownership() -> Bucket {
            let (mint_badge, resource_address, bucket) = Self::create_non_fungible_mutable();

            let vault = Vault::with_bucket(bucket);

            mint_badge.authorize(|| {
                borrow_resource_manager!(resource_address).update_non_fungible_data(
                    &NonFungibleLocalId::integer(0),
                    "own",
                    Some(vault.0),
                );
            });

            mint_badge
        }

        pub fn update_non_fungible(field: String, value: bool) -> (Bucket, Bucket) {
            let (mint_badge, resource_address, bucket) = Self::create_non_fungible_mutable();

            mint_badge.authorize(|| {
                borrow_resource_manager!(resource_address).update_non_fungible_data(
                    &NonFungibleLocalId::integer(0),
                    &field,
                    value,
                );
            });

            (mint_badge, bucket)
        }

        pub fn update_and_get_non_fungible_reference(
            reference: ComponentAddress,
        ) -> (Bucket, Bucket) {
            let (mint_badge, resource_address, bucket) = Self::create_non_fungible_mutable();

            mint_badge.authorize(|| {
                borrow_resource_manager!(resource_address).update_non_fungible_data(
                    &NonFungibleLocalId::integer(0),
                    "reference",
                    Some(reference),
                );
            });

            let data: Sandwich = borrow_resource_manager!(resource_address)
                .get_non_fungible_data(&NonFungibleLocalId::integer(0));
            assert_eq!(data.reference, Some(reference));
            (mint_badge, bucket)
        }

        pub fn update_and_get_non_fungible() -> (Bucket, Bucket) {
            let (mint_badge, resource_address, bucket) = Self::create_non_fungible_mutable();
            let data: Sandwich = borrow_resource_manager!(resource_address)
                .get_non_fungible_data(&NonFungibleLocalId::integer(0));
            assert_eq!(data.available, false);

            mint_badge.authorize(|| {
                borrow_resource_manager!(resource_address).update_non_fungible_data(
                    &NonFungibleLocalId::integer(0),
                    "available",
                    true,
                );
            });

            let data: Sandwich = borrow_resource_manager!(resource_address)
                .get_non_fungible_data(&NonFungibleLocalId::integer(0));
            assert_eq!(data.available, true);
            (mint_badge, bucket)
        }

        pub fn get_total_supply() {
            let resource_address = ResourceBuilder::new_integer_non_fungible::<Sandwich>()
                .metadata("name", "Katz's Sandwiches")
                .create_with_no_initial_supply();

            assert_eq!(
                borrow_resource_manager!(resource_address).total_supply(),
                Decimal::zero(),
            );
        }

        pub fn non_fungible_exists() -> (Bucket, Bucket) {
            let (mint_badge, resource_address, bucket) = Self::create_non_fungible_mutable();
            assert_eq!(
                borrow_resource_manager!(resource_address)
                    .non_fungible_exists(&NonFungibleLocalId::integer(0)),
                true
            );
            assert_eq!(
                borrow_resource_manager!(resource_address)
                    .non_fungible_exists(&NonFungibleLocalId::integer(1)),
                false
            );
            (mint_badge, bucket)
        }

        pub fn take_and_put_bucket() -> Bucket {
            let mut bucket = Self::create_non_fungible_fixed();
            assert_eq!(bucket.amount(), 3.into());

            let non_fungible = bucket.take(1);
            assert_eq!(bucket.amount(), 2.into());
            assert_eq!(non_fungible.amount(), 1.into());

            bucket.put(non_fungible);
            bucket
        }

        pub fn take_non_fungible_and_put_bucket() -> Bucket {
            let mut bucket = Self::create_non_fungible_fixed();
            assert_eq!(bucket.amount(), 3.into());

            let non_fungible = bucket.take_non_fungible(&NonFungibleLocalId::integer(1));
            assert_eq!(bucket.amount(), 2.into());
            assert_eq!(non_fungible.amount(), 1.into());

            bucket.put(non_fungible);
            bucket
        }

        pub fn take_non_fungibles_and_put_bucket() -> Bucket {
            let mut bucket = Self::create_non_fungible_fixed();
            assert_eq!(bucket.amount(), 3.into());

            let mut non_fungibles = BTreeSet::new();
            non_fungibles.insert(NonFungibleLocalId::integer(1));

            let non_fungible = bucket.take_non_fungibles(&non_fungibles);
            assert_eq!(bucket.amount(), 2.into());
            assert_eq!(non_fungible.amount(), 1.into());

            bucket.put(non_fungible);
            bucket
        }

        pub fn take_and_put_vault() -> Bucket {
            let mut vault = Vault::with_bucket(Self::create_non_fungible_fixed());
            assert_eq!(vault.amount(), 3.into());

            let non_fungible = vault.take(1);
            assert_eq!(vault.amount(), 2.into());
            assert_eq!(non_fungible.amount(), 1.into());

            NonFungibleTest { vault }.instantiate().globalize();

            non_fungible
        }

        pub fn get_non_fungible_local_ids_bucket() -> (Bucket, Bucket) {
            let mut bucket = Self::create_non_fungible_fixed();
            let non_fungible_bucket = bucket.take(1);
            assert_eq!(
                non_fungible_bucket.non_fungible_local_ids(),
                BTreeSet::from([NonFungibleLocalId::integer(1)])
            );
            assert_eq!(
                bucket.non_fungible_local_ids(),
                BTreeSet::from([
                    NonFungibleLocalId::integer(2),
                    NonFungibleLocalId::integer(3)
                ])
            );
            (bucket, non_fungible_bucket)
        }

        pub fn get_non_fungible_local_id_bucket() -> (Bucket, Bucket) {
            let mut bucket = Self::create_non_fungible_fixed();
            let non_fungible_bucket = bucket.take(1);
            assert_eq!(
                non_fungible_bucket.non_fungible_local_id(),
                NonFungibleLocalId::integer(1)
            );
            assert_eq!(
                bucket.non_fungible_local_id(),
                NonFungibleLocalId::integer(2)
            );
            (bucket, non_fungible_bucket)
        }

        pub fn get_non_fungible_local_ids_vault() -> Bucket {
            let mut vault = Vault::with_bucket(Self::create_non_fungible_fixed());
            let non_fungible_bucket = vault.take(1);
            assert_eq!(
                non_fungible_bucket.non_fungible_local_ids(),
                BTreeSet::from([NonFungibleLocalId::integer(1)])
            );
            assert_eq!(
                vault.non_fungible_local_ids(),
                BTreeSet::from([
                    NonFungibleLocalId::integer(2),
                    NonFungibleLocalId::integer(3)
                ])
            );

            NonFungibleTest { vault }.instantiate().globalize();

            non_fungible_bucket
        }

        pub fn get_non_fungible_local_id_vault() -> Bucket {
            let mut vault = Vault::with_bucket(Self::create_non_fungible_fixed());
            let non_fungible_bucket = vault.take(1);
            assert_eq!(
                non_fungible_bucket.non_fungible_local_id(),
                NonFungibleLocalId::integer(1)
            );
            assert_eq!(
                vault.non_fungible_local_id(),
                NonFungibleLocalId::integer(2)
            );

            NonFungibleTest { vault }.instantiate().globalize();

            non_fungible_bucket
        }

        pub fn singleton_non_fungible() {
            let mut bucket = Self::create_non_fungible_fixed();
            assert_eq!(bucket.amount(), 3.into());

            // read singleton bucket
            let singleton = bucket.take(1);
            let _: Sandwich = singleton.non_fungible().data();

            // read singleton vault
            let mut vault = Vault::with_bucket(singleton);
            let _: Sandwich = vault.non_fungible().data();

            // read singleton proof
            let proof = vault.create_proof();
            let validated_proof = proof.validate_proof(vault.resource_address()).unwrap();
            let _: Sandwich = validated_proof.non_fungible().data();
            validated_proof.drop();

            // clean up
            vault.put(bucket);
            NonFungibleTest { vault }.instantiate().globalize();
        }

        pub fn create_wrong_non_fungible_local_id_type() -> Bucket {
            let mut entries = BTreeMap::new();
            entries.insert(
                NonFungibleLocalId::integer(0),
                (scrypto_decode(&scrypto_encode(&()).unwrap()).unwrap(),),
            );

            // creating non-fungible id with id type set to default (UUID)
            let rtn = ScryptoEnv
                .call_function(
                    RESOURCE_MANAGER_PACKAGE,
                    NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
                    scrypto_encode(&NonFungibleResourceManagerCreateWithInitialSupplyInput {
                        id_type: NonFungibleIdType::UUID,
                        metadata: BTreeMap::new(),
                        access_rules: BTreeMap::new(),
                        non_fungible_schema: NonFungibleDataSchema::new_schema::<()>(),
                        entries,
                    })
                    .unwrap(),
                )
                .unwrap();
            let (_resource_address, bucket): (ResourceAddress, Bucket) =
                scrypto_decode(&rtn).unwrap();

            bucket
        }

        pub fn create_string_non_fungible() -> Bucket {
            // creating non-fungible id with id type set to default (UUID)
            ResourceBuilder::new_string_non_fungible::<Sandwich>().mint_initial_supply([
                (
                    "1".try_into().unwrap(),
                    Sandwich {
                        name: "One".to_owned(),
                        available: true,
                        tastes_great: true,
                        reference: None,
                        own: None,
                    },
                ),
                (
                    "2".try_into().unwrap(),
                    Sandwich {
                        name: "Two".to_owned(),
                        available: true,
                        tastes_great: true,
                        reference: None,
                        own: None,
                    },
                ),
            ])
        }

        pub fn create_bytes_non_fungible() -> Bucket {
            ResourceBuilder::new_bytes_non_fungible::<Sandwich>().mint_initial_supply([
                (
                    1u32.to_le_bytes().to_vec().try_into().unwrap(),
                    Sandwich {
                        name: "One".to_owned(),
                        available: true,
                        tastes_great: true,
                        reference: None,
                        own: None,
                    },
                ),
                (
                    2u32.to_le_bytes().to_vec().try_into().unwrap(),
                    Sandwich {
                        name: "Two".to_owned(),
                        available: true,
                        tastes_great: true,
                        reference: None,
                        own: None,
                    },
                ),
            ])
        }

        pub fn create_uuid_non_fungible() -> Bucket {
            ResourceBuilder::new_uuid_non_fungible::<Sandwich>().mint_initial_supply([Sandwich {
                name: "Zero".to_owned(),
                available: true,
                tastes_great: true,
                reference: None,
                own: None,
            }])
        }

        pub fn create_mintable_uuid_non_fungible() -> ResourceAddress {
            ResourceBuilder::new_uuid_non_fungible::<Sandwich>()
                .mintable(rule!(allow_all), rule!(deny_all))
                .create_with_no_initial_supply()
        }

        pub fn create_uuid_non_fungible_and_mint() -> Bucket {
            // creating non-fungible id with id type set to default (UUID)
            let resource_address = ResourceBuilder::new_uuid_non_fungible::<Sandwich>()
                .mintable(rule!(allow_all), rule!(deny_all))
                .metadata("name", "Katz's Sandwiches")
                .create_with_no_initial_supply();

            borrow_resource_manager!(resource_address).mint_uuid_non_fungible(Sandwich {
                name: "Test".to_owned(),
                available: false,
                tastes_great: true,
                reference: None,
                own: None,
            })
        }
    }
}
