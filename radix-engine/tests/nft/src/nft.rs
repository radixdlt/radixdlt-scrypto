use scrypto::prelude::*;

blueprint! {
    struct NftTest;

    impl NftTest {
        pub fn create_nft_mutable() -> (Bucket, ResourceDef, Bucket) {
            // Create a mint badge
            let mint_badge = ResourceBuilder::new_fungible(18)
            .flags(FREELY_TRANSFERABLE | FREELY_BURNABLE)
                .initial_supply(NewSupply::fungible(1));

            // Create NFT resource with mutable supply
            let nft_resource_def = ResourceBuilder::new_non_fungible()
                .metadata("name", "Katz's Sandwiches")
                .flags(FREELY_TRANSFERABLE | MINTABLE | BURNABLE | INDIVIDUAL_METADATA_MUTABLE)
                .badge(mint_badge.resource_def(), MAY_MINT | MAY_BURN | MAY_CHANGE_INDIVIDUAL_METADATA)
                .no_initial_supply();

            // Mint an NFT
            let nft = nft_resource_def.mint_nft(0, "Prastrami", mint_badge.present());

            (mint_badge, nft_resource_def, nft)
        }

        pub fn create_nft_fixed() -> Bucket {
            ResourceBuilder::new_non_fungible()
                .metadata("name", "Katz's Sandwiches")
                .flags(FREELY_TRANSFERABLE | FREELY_BURNABLE)
                .initial_supply(NewSupply::non_fungible([
                    (1, "Hi"),
                    (2, "Test"),
                    (3, "NFT"),
                ]))
        }

        pub fn update_and_get_nft() -> (Bucket, Bucket) {
            let (mint_badge, resource_def, bucket) = Self::create_nft_mutable();
            let nft: String = resource_def.get_nft_data(0);
            assert_eq!(nft, "Prastrami");
            resource_def.update_nft_data(0, "New String", mint_badge.present());
            let nft: String = resource_def.get_nft_data(0);
            assert_eq!(nft, "New String");
            (mint_badge, bucket)
        }

        pub fn take_and_put_bucket() -> Bucket {
            let bucket = Self::create_nft_fixed();
            assert_eq!(bucket.amount(), 3.into());

            let nft = bucket.take(1);
            assert_eq!(bucket.amount(), 2.into());
            assert_eq!(nft.amount(), 1.into());

            bucket.put(nft);
            bucket
        }

        pub fn take_and_put_vault() -> Bucket {
            let vault = Vault::with_bucket(Self::create_nft_fixed());
            assert_eq!(vault.amount(), 3.into());

            let nft = vault.take(1);
            assert_eq!(vault.amount(), 2.into());
            assert_eq!(nft.amount(), 1.into());

            nft
        }

        pub fn get_nft_ids_bucket() -> (Bucket, Bucket) {
            let bucket = Self::create_nft_fixed();
            let nft = bucket.take(1);
            assert_eq!(bucket.get_nft_ids(), BTreeSet::from([2, 3]));
            assert_eq!(nft.get_nft_ids(), BTreeSet::from([1]));
            (bucket, nft)
        }

        pub fn get_nft_ids_vault() -> Bucket {
            let vault = Vault::with_bucket(Self::create_nft_fixed());
            let nft = vault.take(1);
            assert_eq!(vault.get_nft_ids(), BTreeSet::from([2, 3]));
            assert_eq!(nft.get_nft_ids(), BTreeSet::from([1]));
            nft
        }
    }
}
