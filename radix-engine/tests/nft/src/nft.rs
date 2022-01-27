use scrypto::prelude::*;

#[derive(NftData)]
pub struct Sandwich {
    pub name: String,
    #[scrypto(mutable)]
    pub available: bool,
}

#[derive(NftData)]
pub struct VaultContainer {
    pub vault: Vault,
}

blueprint! {
    struct NftTest;

    impl NftTest {
        pub fn create_nft_mutable() -> (Bucket, ResourceDef, Bucket) {
            // Create a mint badge
            let mint_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE).initial_supply_fungible(1);

            // Create NFT resource with mutable supply
            let mut nft_resource_def = ResourceBuilder::new_non_fungible()
                .metadata("name", "Katz's Sandwiches")
                .flags(MINTABLE | BURNABLE | INDIVIDUAL_METADATA_MUTABLE)
                .badge(
                    mint_badge.resource_def(),
                    MAY_MINT | MAY_BURN | MAY_CHANGE_INDIVIDUAL_METADATA,
                )
                .no_initial_supply();

            // Mint an NFT
            let nft = nft_resource_def.mint_nft(
                0,
                Sandwich {
                    name: "Test".to_owned(),
                    available: false,
                },
                mint_badge.present(),
            );

            (mint_badge, nft_resource_def, nft)
        }

        pub fn create_nft_fixed() -> Bucket {
            ResourceBuilder::new_non_fungible()
                .metadata("name", "Katz's Sandwiches")
                .initial_supply_non_fungible([
                    (
                        1,
                        Sandwich {
                            name: "One".to_owned(),
                            available: true,
                        },
                    ),
                    (
                        2,
                        Sandwich {
                            name: "Two".to_owned(),
                            available: true,
                        },
                    ),
                    (
                        3,
                        Sandwich {
                            name: "Three".to_owned(),
                            available: true,
                        },
                    ),
                ])
        }

        pub fn update_and_get_nft() -> (Bucket, Bucket) {
            let (mint_badge, mut resource_def, bucket) = Self::create_nft_mutable();
            let mut data: Sandwich = resource_def.get_nft_data(0);
            assert_eq!(data.available, false);

            data.available = true;
            resource_def.update_nft_data(0, data, mint_badge.present());

            let data: Sandwich = resource_def.get_nft_data(0);
            assert_eq!(data.available, true);
            (mint_badge, bucket)
        }

        pub fn take_and_put_bucket() -> Bucket {
            let mut bucket = Self::create_nft_fixed();
            assert_eq!(bucket.amount(), 3.into());

            let nft = bucket.take(1);
            assert_eq!(bucket.amount(), 2.into());
            assert_eq!(nft.amount(), 1.into());

            bucket.put(nft);
            bucket
        }

        pub fn take_and_put_vault() -> Bucket {
            let mut vault = Vault::with_bucket(Self::create_nft_fixed());
            assert_eq!(vault.amount(), 3.into());

            let nft = vault.take(1);
            assert_eq!(vault.amount(), 2.into());
            assert_eq!(nft.amount(), 1.into());

            nft
        }

        pub fn get_nft_ids_bucket() -> (Bucket, Bucket) {
            let mut bucket = Self::create_nft_fixed();
            let nft = bucket.take(1);
            assert_eq!(bucket.get_nft_ids(), Vec::from([2, 3]));
            assert_eq!(nft.get_nft_ids(), Vec::from([1]));
            (bucket, nft)
        }

        pub fn get_nft_ids_vault() -> Bucket {
            let mut vault = Vault::with_bucket(Self::create_nft_fixed());
            let nft = vault.take(1);
            assert_eq!(vault.get_nft_ids(), Vec::from([2, 3]));
            assert_eq!(nft.get_nft_ids(), Vec::from([1]));
            nft
        }

        pub fn nft_and_vault() -> (Bucket, Bucket) {
            let (mint_badge, mut resource_def, mut bucket) = Self::create_nft_mutable();

            let nft = mint_badge.authorize(|auth| {
                resource_def.mint_nft(
                    Uuid::generate(),
                    VaultContainer {
                        vault: Vault::new(RADIX_TOKEN),
                    },
                    auth,
                )
            });
            bucket.put(nft);

            (mint_badge, bucket)
        }
    }
}
