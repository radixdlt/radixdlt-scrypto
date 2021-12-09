use scrypto::prelude::*;

blueprint! {
    struct BucketTest;

    impl BucketTest {
        fn create_test_token(amount: u32) -> Bucket {
            ResourceBuilder::new_fungible(0)
                .metadata("name", "TestToken")
                .initial_supply_fungible(amount)
        }

        pub fn combine() -> Bucket {
            let bucket1 = Self::create_test_token(100);
            let bucket2 = bucket1.take(50);

            bucket1.put(bucket2);
            bucket1
        }

        pub fn split() -> (Bucket, Bucket) {
            let bucket1 = Self::create_test_token(100);
            let bucket2 = bucket1.take(Decimal::from(5));
            (bucket1, bucket2)
        }

        pub fn borrow() -> Bucket {
            let bucket = Self::create_test_token(100);
            let bucket_ref = bucket.present();
            bucket_ref.drop();
            bucket
        }

        pub fn query() -> (Decimal, Address, Bucket) {
            let bucket = Self::create_test_token(100);
            (bucket.amount(), bucket.resource_address(), bucket)
        }

        pub fn test_restricted_transfer() -> Vec<Bucket> {
            let badge = ResourceBuilder::new_fungible(18).initial_supply_fungible(1);
            let bucket = ResourceBuilder::new_fungible(0)
                .flags(RESTRICTED_TRANSFER)
                .badge(badge.resource_address(), MAY_TRANSFER)
                .initial_supply_fungible(5);
            let vault = Vault::with_bucket(bucket);
            let bucket2 = vault.take_with_auth(1, badge.present());
            vec![badge, bucket2]
        }

        pub fn test_burn() -> Vec<Bucket> {
            let badge = ResourceBuilder::new_fungible(18).initial_supply_fungible(1);
            let bucket = ResourceBuilder::new_fungible(0)
                .flags(BURNABLE)
                .badge(badge.resource_address(), MAY_BURN)
                .initial_supply_fungible(5);
            bucket.burn(Some(badge.present()));
            vec![badge]
        }

        pub fn test_burn_freely() -> Vec<Bucket> {
            let badge = ResourceBuilder::new_fungible(18).initial_supply_fungible(1);
            let bucket1 = ResourceBuilder::new_fungible(0)
                .flags(FREELY_BURNABLE)
                .initial_supply_fungible(5);
            let bucket2 = bucket1.take(2);
            bucket1.burn(Some(badge.present()));
            bucket2.burn(None);
            vec![badge]
        }
    }
}
