use scrypto::prelude::*;

blueprint! {
    struct ResourceTest;

    impl ResourceTest {
        pub fn create_mutable_token() -> (Bucket, ResourceDef) {
            let badge = ResourceBuilder::new()
                .metadata("name", "Auth")
                .new_badge_fixed(1);
            let resource_def = ResourceBuilder::new()
                .metadata("name", "TestToken")
                .new_token_mutable(ResourceAuthConfigs::new(badge.resource_def()));
            (badge, resource_def)
        }

        pub fn create_fixed_token() -> (Bucket, Bucket) {
            let bucket = ResourceBuilder::new()
                .metadata("name", "TestToken")
                .new_token_fixed(100);
            (
                bucket.take(Decimal::from_str("0.000000000000000001").unwrap()),
                bucket,
            )
        }

        pub fn create_mutable_badge() -> (Bucket, ResourceDef) {
            let badge = ResourceBuilder::new()
                .metadata("name", "Auth")
                .new_badge_fixed(1);
            let resource_def = ResourceBuilder::new()
                .metadata("name", "TestToken")
                .new_badge_mutable(ResourceAuthConfigs::new(badge.resource_def()));
            (badge, resource_def)
        }

        pub fn create_fixed_badge() -> (Bucket, Bucket) {
            let bucket = ResourceBuilder::new()
                .metadata("name", "TestToken")
                .new_badge_fixed(100);

            (bucket.take(1), bucket)
        }

        pub fn create_fixed_badge_should_fail() -> (Bucket, Bucket) {
            let bucket = ResourceBuilder::new()
                .metadata("name", "TestToken")
                .new_badge_fixed(100);

            (bucket.take(Decimal::from_str("0.1").unwrap()), bucket)
        }

        pub fn query() -> (Bucket, HashMap<String, String>, Option<ResourceAuthConfigs>, Decimal) {
            let bucket = ResourceBuilder::new()
                .metadata("name", "TestToken")
                .new_token_fixed(100);
            let resource_def = bucket.resource_def();
            (
                bucket,
                resource_def.metadata(),
                resource_def.auth_configs(),
                resource_def.total_supply(),
            )
        }

        pub fn burn() -> Bucket {
            let (auth, resource_def) = Self::create_mutable_token();
            let bucket = resource_def.mint(1, auth.borrow());
            resource_def.burn(bucket, auth.borrow());
            auth
        }

        pub fn change_to_immutable() -> Bucket {
            let (auth, resource_def) = Self::create_mutable_token();
            resource_def.change_to_immutable(auth.borrow());
            auth
        }
    }
}
