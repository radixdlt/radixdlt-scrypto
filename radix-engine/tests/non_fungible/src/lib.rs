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
                .auth("take_from_vault", auth!(allow_all))
                .initial_supply(1);

            // Create non-fungible resource with mutable supply
            let resource_address = ResourceBuilder::new_non_fungible()
                .metadata("name", "Katz's Sandwiches")
                .auth("take_from_vault", auth!(allow_all))
                .auth("mint", auth!(require(mint_badge.resource_address())))
                .auth("burn", auth!(require(mint_badge.resource_address())))
                .auth(
                    "update_non_fungible_mutable_data",
                    auth!(require(mint_badge.resource_address())),
                )
                .no_initial_supply();

            // Mint a non-fungible
            let non_fungible = mint_badge.authorize(|| {
                resource_manager!(resource_address).mint_non_fungible(
                    &NonFungibleId::from_u32(0),
                    Sandwich {
                        name: "Test".to_owned(),
                        available: false,
                    },
                )
            });

            (mint_badge, resource_address, non_fungible)
        }

        pub fn create_non_fungible_fixed() -> Bucket {
            ResourceBuilder::new_non_fungible()
                .metadata("name", "Katz's Sandwiches")
                .auth("take_from_vault", auth!(allow_all))
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

        pub fn update_and_get_non_fungible() -> (Bucket, Bucket) {
            let (mint_badge, resource_address, bucket) = Self::create_non_fungible_mutable();
            let mut data: Sandwich = resource_manager!(resource_address)
                .get_non_fungible_data(&NonFungibleId::from_u32(0));
            assert_eq!(data.available, false);

            data.available = true;
            mint_badge.authorize(|| {
                resource_manager!(resource_address)
                    .update_non_fungible_data(&NonFungibleId::from_u32(0), data);
            });

            let data: Sandwich = resource_manager!(resource_address)
                .get_non_fungible_data(&NonFungibleId::from_u32(0));
            assert_eq!(data.available, true);
            (mint_badge, bucket)
        }

        pub fn non_fungible_exists() -> (Bucket, Bucket) {
            let (mint_badge, resource_address, bucket) = Self::create_non_fungible_mutable();
            assert_eq!(
                resource_manager!(resource_address)
                    .non_fungible_exists(&NonFungibleId::from_u32(0)),
                true
            );
            assert_eq!(
                resource_manager!(resource_address)
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
    }
}
