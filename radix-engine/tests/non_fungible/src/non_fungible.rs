use scrypto::prelude::*;

#[derive(NonFungibleData)]
pub struct Sandwich {
    pub name: String,
    #[scrypto(mutable)]
    pub available: bool,
}

blueprint! {
    struct NonFungibleTest {
        vault: Vault
    }

    impl NonFungibleTest {
        pub fn create_non_fungible_mutable() -> (Bucket, ResourceDef, Bucket) {
            // Create a mint badge
            let mint_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE).initial_supply_fungible(1);

            // Create non-fungible resource with mutable supply
            let mut non_fungible_resource_def = ResourceBuilder::new_non_fungible()
                .metadata("name", "Katz's Sandwiches")
                .flags(MINTABLE | BURNABLE | INDIVIDUAL_METADATA_MUTABLE)
                .badge(
                    mint_badge.resource_def(),
                    MAY_MINT | MAY_BURN | MAY_CHANGE_INDIVIDUAL_METADATA,
                )
                .no_initial_supply();

            // Mint a non-fungible
            let non_fungible = non_fungible_resource_def.mint_non_fungible(
                &NonFungibleKey::from(0u128),
                Sandwich {
                    name: "Test".to_owned(),
                    available: false,
                },
                mint_badge.present(),
            );

            (mint_badge, non_fungible_resource_def, non_fungible)
        }

        pub fn create_non_fungible_fixed() -> Bucket {
            ResourceBuilder::new_non_fungible()
                .metadata("name", "Katz's Sandwiches")
                .initial_supply_non_fungible([
                    (
                        NonFungibleKey::from(1u128),
                        Sandwich {
                            name: "One".to_owned(),
                            available: true,
                        },
                    ),
                    (
                        NonFungibleKey::from(2u128),
                        Sandwich {
                            name: "Two".to_owned(),
                            available: true,
                        },
                    ),
                    (
                        NonFungibleKey::from(3u128),
                        Sandwich {
                            name: "Three".to_owned(),
                            available: true,
                        },
                    ),
                ])
        }

        pub fn update_and_get_non_fungible() -> (Bucket, Bucket) {
            let (mint_badge, mut resource_def, bucket) = Self::create_non_fungible_mutable();
            let mut data: Sandwich = resource_def.get_non_fungible_data(&NonFungibleKey::from(0u128));
            assert_eq!(data.available, false);

            data.available = true;
            resource_def.update_non_fungible_data(&NonFungibleKey::from(0u128), data, mint_badge.present());

            let data: Sandwich = resource_def.get_non_fungible_data(&NonFungibleKey::from(0u128));
            assert_eq!(data.available, true);
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

            NonFungibleTest { vault }.instantiate();

            non_fungible
        }

        pub fn get_non_fungible_ids_bucket() -> (Bucket, Bucket) {
            let mut bucket = Self::create_non_fungible_fixed();
            let non_fungible = bucket.take(1);
            assert_eq!(bucket.get_non_fungible_keys(), Vec::from([NonFungibleKey::from(2u128), NonFungibleKey::from(3u128)]));
            assert_eq!(non_fungible.get_non_fungible_keys(), Vec::from([NonFungibleKey::from(1u128)]));
            (bucket, non_fungible)
        }

        pub fn get_non_fungible_ids_vault() -> Bucket {
            let mut vault = Vault::with_bucket(Self::create_non_fungible_fixed());
            let non_fungible = vault.take(1);
            assert_eq!(vault.get_non_fungible_keys(), Vec::from([NonFungibleKey::from(2u128), NonFungibleKey::from(3u128)]));
            assert_eq!(non_fungible.get_non_fungible_keys(), Vec::from([NonFungibleKey::from(1u128)]));

            NonFungibleTest { vault }.instantiate();

            non_fungible
        }

    }
}
