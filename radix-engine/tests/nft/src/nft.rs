use scrypto::prelude::*;

blueprint! {
    struct NftTest;

    impl NftTest {
        pub fn create_nft_mutable() -> (Bucket, ResourceDef, Bucket) {
            let minter_badge = ResourceBuilder::new().new_badge_fixed(1);

            let resource_def = ResourceBuilder::new()
                .metadata("name", "Katz's Sandwiches")
                .new_nft_mutable(minter_badge.resource_address());

            let nft = resource_def.mint_nft(0, "Prastrami", minter_badge.borrow());

            (minter_badge, resource_def, nft)
        }

        pub fn create_nft_fixed() -> Bucket {
            ResourceBuilder::new()
                .metadata("name", "Katz's Sandwiches")
                .new_nft_fixed(BTreeMap::from([
                    (1, "Hi"), 
                    (2, "Test"), 
                    (3, "NFT")
                ]))
        }

        pub fn update_and_get_nft() -> (Bucket, Bucket) {
            let (minter, resource_def, bucket) = Self::create_nft_mutable();
            let nft: String = resource_def.get_nft_data(0);
            assert_eq!(nft, "Prastrami");
            resource_def.update_nft_data(0, "New String", minter.borrow());
            let nft: String = resource_def.get_nft_data(0);
            assert_eq!(nft, "New String");
            (minter, bucket)
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
