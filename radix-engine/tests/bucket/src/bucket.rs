use scrypto::prelude::*;

blueprint! {
    struct BucketTest;

    impl BucketTest {
        fn create_test_token(amount: u32) -> Bucket {
            ResourceBuilder::new_fungible(0)
                .metadata("name", "TestToken")
                .flags(FREELY_TRANSFERABLE | FREELY_BURNABLE)
                .initial_supply(NewSupply::fungible(amount))
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
    }
}
