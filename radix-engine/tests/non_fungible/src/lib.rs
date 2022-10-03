use scrypto::prelude::*;

#[derive(NonFungibleData)]
pub struct Sandwich {
    pub name: String,
    #[scrypto(mutable)]
    pub available: bool,
}

blueprint! {
    struct NonFungibleTest {
        vault: Vault,
    }

    impl NonFungibleTest {
        pub fn create_non_fungible_mutable() -> (Bucket, ResourceAddress, Bucket) {
            // Create a mint badge
            let mint_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply(1);

            // Create non-fungible resource with mutable supply
            let resource_address = ResourceBuilder::new_non_fungible()
                .metadata("name", "Katz's Sandwiches")
                .mintable(rule!(require(mint_badge.resource_address())), LOCKED)
                .burnable(rule!(allow_all), LOCKED)
                .updateable_non_fungible_data(rule!(require(mint_badge.resource_address())), LOCKED)
                .no_initial_supply();

            // Mint a non-fungible
            let non_fungible = mint_badge.authorize(|| {
                borrow_resource_manager!(resource_address).mint_non_fungible(
                    &NonFungibleId::from_u32(0),
                    Sandwich {
                        name: "Test".to_owned(),
                        available: false,
                    },
                )
            });

            (mint_badge, resource_address, non_fungible)
        }

        pub fn update_nft(mint_badge: Bucket, proof: Proof) -> Bucket {
            let proof = proof.unsafe_skip_proof_validation();
            mint_badge.authorize(|| {
                borrow_resource_manager!(proof.resource_address()).update_non_fungible_data(
                    &proof.non_fungible_id(),
                    Sandwich {
                        name: "Test".to_owned(),
                        available: true,
                    },
                )
            });

            mint_badge
        }

        pub fn create_burnable_non_fungible() -> Bucket {
            ResourceBuilder::new_non_fungible()
                .metadata("name", "Katz's Sandwiches")
                .burnable(rule!(allow_all), LOCKED)
                .initial_supply([
                    (
                        NonFungibleId::from_u32(0),
                        Sandwich {
                            name: "Zero".to_owned(),
                            available: true,
                        },
                    ),
                    (
                        NonFungibleId::from_u32(1),
                        Sandwich {
                            name: "One".to_owned(),
                            available: true,
                        },
                    ),
                ])
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

        pub fn verify_does_not_exist(address: NonFungibleAddress) {
            assert_eq!(
                borrow_resource_manager!(address.resource_address())
                    .non_fungible_exists(&address.non_fungible_id()),
                false
            );
        }

        pub fn update_and_get_non_fungible() -> (Bucket, Bucket) {
            let (mint_badge, resource_address, bucket) = Self::create_non_fungible_mutable();
            let mut data: Sandwich = borrow_resource_manager!(resource_address)
                .get_non_fungible_data(&NonFungibleId::from_u32(0));
            assert_eq!(data.available, false);

            data.available = true;
            mint_badge.authorize(|| {
                borrow_resource_manager!(resource_address)
                    .update_non_fungible_data(&NonFungibleId::from_u32(0), data);
            });

            let data: Sandwich = borrow_resource_manager!(resource_address)
                .get_non_fungible_data(&NonFungibleId::from_u32(0));
            assert_eq!(data.available, true);
            (mint_badge, bucket)
        }

        pub fn non_fungible_exists() -> (Bucket, Bucket) {
            let (mint_badge, resource_address, bucket) = Self::create_non_fungible_mutable();
            assert_eq!(
                borrow_resource_manager!(resource_address)
                    .non_fungible_exists(&NonFungibleId::from_u32(0)),
                true
            );
            assert_eq!(
                borrow_resource_manager!(resource_address)
                    .non_fungible_exists(&NonFungibleId::from_u32(1)),
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

        pub fn take_and_put_vault() -> Bucket {
            let mut vault = Vault::with_bucket(Self::create_non_fungible_fixed());
            assert_eq!(vault.amount(), 3.into());

            let non_fungible = vault.take(1);
            assert_eq!(vault.amount(), 2.into());
            assert_eq!(non_fungible.amount(), 1.into());

            NonFungibleTest { vault }.instantiate().globalize();

            non_fungible
        }

        pub fn get_non_fungible_ids_bucket() -> (Bucket, Bucket) {
            let mut bucket = Self::create_non_fungible_fixed();
            let non_fungible_bucket = bucket.take(1);
            assert_eq!(
                non_fungible_bucket.non_fungible_ids(),
                BTreeSet::from([NonFungibleId::from_u32(1)])
            );
            assert_eq!(
                bucket.non_fungible_ids(),
                BTreeSet::from([NonFungibleId::from_u32(2), NonFungibleId::from_u32(3)])
            );
            (bucket, non_fungible_bucket)
        }

        pub fn get_non_fungible_id_bucket() -> (Bucket, Bucket) {
            let mut bucket = Self::create_non_fungible_fixed();
            let non_fungible_bucket = bucket.take(1);
            assert_eq!(
                non_fungible_bucket.non_fungible_id(),
                NonFungibleId::from_u32(1)
            );
            assert_eq!(bucket.non_fungible_id(), NonFungibleId::from_u32(2));
            (bucket, non_fungible_bucket)
        }

        pub fn get_non_fungible_ids_vault() -> Bucket {
            let mut vault = Vault::with_bucket(Self::create_non_fungible_fixed());
            let non_fungible_bucket = vault.take(1);
            assert_eq!(
                non_fungible_bucket.non_fungible_ids(),
                BTreeSet::from([NonFungibleId::from_u32(1)])
            );
            assert_eq!(
                vault.non_fungible_ids(),
                BTreeSet::from([NonFungibleId::from_u32(2), NonFungibleId::from_u32(3)])
            );

            NonFungibleTest { vault }.instantiate().globalize();

            non_fungible_bucket
        }

        pub fn get_non_fungible_id_vault() -> Bucket {
            let mut vault = Vault::with_bucket(Self::create_non_fungible_fixed());
            let non_fungible_bucket = vault.take(1);
            assert_eq!(
                non_fungible_bucket.non_fungible_id(),
                NonFungibleId::from_u32(1)
            );
            assert_eq!(vault.non_fungible_id(), NonFungibleId::from_u32(2));

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

            // clean up
            vault.put(bucket);
            NonFungibleTest { vault }.instantiate().globalize();
        }
    }
}
