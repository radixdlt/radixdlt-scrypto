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
        pub fn create_non_fungible_mutable() -> (Bucket, ResourceDefId, Bucket) {
            // Create a mint badge
            let mint_badge =
                ResourceBuilder::new_fungible(DIVISIBILITY_NONE).initial_supply_fungible(1);

            // Create non-fungible resource with mutable supply
            let resource_def_id = ResourceBuilder::new_non_fungible()
                .metadata("name", "Katz's Sandwiches")
                .flags(MINTABLE | BURNABLE | INDIVIDUAL_METADATA_MUTABLE)
                .badge(
                    mint_badge.resource_def_id(),
                    MAY_MINT | MAY_BURN | MAY_CHANGE_INDIVIDUAL_METADATA,
                )
                .no_initial_supply();

            // Mint a non-fungible
            let non_fungible = mint_badge.authorize(|| {
                resource_def!(resource_def_id).mint_non_fungible(
                    &NonFungibleId::from(0u128),
                    Sandwich {
                        name: "Test".to_owned(),
                        available: false,
                    },
                )
            });

            (mint_badge, resource_def_id, non_fungible)
        }

        pub fn create_non_fungible_fixed() -> Bucket {
            ResourceBuilder::new_non_fungible()
                .metadata("name", "Katz's Sandwiches")
                .initial_supply_non_fungible([
                    (
                        NonFungibleId::from(1u128),
                        Sandwich {
                            name: "One".to_owned(),
                            available: true,
                        },
                    ),
                    (
                        NonFungibleId::from(2u128),
                        Sandwich {
                            name: "Two".to_owned(),
                            available: true,
                        },
                    ),
                    (
                        NonFungibleId::from(3u128),
                        Sandwich {
                            name: "Three".to_owned(),
                            available: true,
                        },
                    ),
                ])
        }

        pub fn update_and_get_non_fungible() -> (Bucket, Bucket) {
            let (mint_badge, resource_def_id, bucket) = Self::create_non_fungible_mutable();
            let mut data: Sandwich =
                resource_def!(resource_def_id).get_non_fungible_data(&NonFungibleId::from(0u128));
            assert_eq!(data.available, false);

            data.available = true;
            mint_badge.authorize(|| {
                resource_def!(resource_def_id)
                    .update_non_fungible_data(&NonFungibleId::from(0u128), data);
            });

            let data: Sandwich =
                resource_def!(resource_def_id).get_non_fungible_data(&NonFungibleId::from(0u128));
            assert_eq!(data.available, true);
            (mint_badge, bucket)
        }

        pub fn non_fungible_exists() -> (Bucket, Bucket) {
            let (mint_badge, resource_def_id, bucket) = Self::create_non_fungible_mutable();
            assert_eq!(
                resource_def!(resource_def_id).non_fungible_exists(&NonFungibleId::from(0u128)),
                true
            );
            assert_eq!(
                resource_def!(resource_def_id).non_fungible_exists(&NonFungibleId::from(1u128)),
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

            NonFungibleTest { vault }.globalize_noauth();

            non_fungible
        }

        pub fn get_non_fungible_ids_bucket() -> (Bucket, Bucket) {
            let mut bucket = Self::create_non_fungible_fixed();
            let non_fungible = bucket.take(1);
            assert_eq!(
                bucket.get_non_fungible_ids(),
                BTreeSet::from([NonFungibleId::from(2u128), NonFungibleId::from(3u128)])
            );
            assert_eq!(
                non_fungible.get_non_fungible_ids(),
                BTreeSet::from([NonFungibleId::from(1u128)])
            );
            (bucket, non_fungible)
        }

        pub fn get_non_fungible_ids_vault() -> Bucket {
            let mut vault = Vault::with_bucket(Self::create_non_fungible_fixed());
            let non_fungible = vault.take(1);
            assert_eq!(
                vault.get_non_fungible_ids(),
                BTreeSet::from([NonFungibleId::from(2u128), NonFungibleId::from(3u128)])
            );
            assert_eq!(
                non_fungible.get_non_fungible_ids(),
                BTreeSet::from([NonFungibleId::from(1u128)])
            );

            NonFungibleTest { vault }.globalize_noauth();

            non_fungible
        }
    }
}

package_init!(blueprint::NonFungibleTest::describe());
