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
                .new_nft_fixed(vec![
                    (1, "Hi"),
                    (2, "Test"),
                    (3, "NFT")
                ])
        }

        pub fn take_and_put() -> Bucket {
            let bucket = Self::create_nft_fixed();
            assert_eq!(bucket.amount(), 3.into());

            let nft = bucket.take(1);
            assert_eq!(bucket.amount(), 2.into());
            assert_eq!(nft.amount(), 1.into());

            bucket.put(nft);
            bucket
        }

        pub fn vault() -> Bucket {
            let vault = Vault::with_bucket(Self::create_nft_fixed());
            assert_eq!(vault.amount(), 3.into());

            let nft = vault.take(1);
            assert_eq!(vault.amount(), 2.into());
            assert_eq!(nft.amount(), 1.into());

            nft
        }
    }
}
