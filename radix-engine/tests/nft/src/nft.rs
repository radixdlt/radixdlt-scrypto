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
                    (2, "Test")
                ])
        }
    }
}
