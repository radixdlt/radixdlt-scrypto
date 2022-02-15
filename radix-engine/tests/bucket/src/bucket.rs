use scrypto::prelude::*;

blueprint! {
    struct BucketTest {
        vault: Vault
    }

    impl BucketTest {
        fn create_test_token(amount: u32) -> Bucket {
            let bucket = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .initial_supply_fungible(amount);
            let ref1 = bucket.present();
            let ref2 = ref1.clone();
            ref1.drop();
            ref2.drop();
            bucket
        }

        pub fn combine() -> Bucket {
            let mut bucket1 = Self::create_test_token(100);
            let bucket2 = bucket1.take(50);

            bucket1.put(bucket2);
            bucket1
        }

        pub fn split() -> (Bucket, Bucket) {
            let mut bucket1 = Self::create_test_token(100);
            let bucket2 = bucket1.take(Decimal::from(5));
            (bucket1, bucket2)
        }

        pub fn borrow() -> Bucket {
            let bucket = Self::create_test_token(100);
            let bucket_ref = bucket.present();
            bucket_ref.drop();
            bucket
        }

        pub fn query() -> (Decimal, ResourceDefRef, Bucket) {
            let bucket = Self::create_test_token(100);
            (bucket.amount(), bucket.resource_def_ref(), bucket)
        }

        pub fn test_restricted_transfer() -> Vec<Bucket> {
            let badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE).initial_supply_fungible(1);
            let bucket = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .flags(RESTRICTED_TRANSFER)
                .badge(badge.resource_def_ref(), MAY_TRANSFER)
                .initial_supply_fungible(5);
            let mut vault = Vault::with_bucket(bucket);
            let bucket2 = vault.take_with_auth(1, badge.present());
            BucketTest { vault }.instantiate();
            vec![badge, bucket2]
        }

        pub fn test_burn() -> Vec<Bucket> {
            let badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE).initial_supply_fungible(1);
            let bucket = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .flags(BURNABLE)
                .badge(badge.resource_def_ref(), MAY_BURN)
                .initial_supply_fungible(5);
            bucket.burn_with_auth(badge.present());
            vec![badge]
        }

        pub fn test_burn_freely() -> Vec<Bucket> {
            let badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE).initial_supply_fungible(1);
            let mut bucket1 = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .flags(BURNABLE | FREELY_BURNABLE)
                .initial_supply_fungible(5);
            let bucket2 = bucket1.take(2);
            bucket1.burn_with_auth(badge.present());
            bucket2.burn();
            vec![badge]
        }
    }
}
