use scrypto::prelude::*;

blueprint! {
    struct BadgeTest;

    impl BadgeTest {
        fn create_test_badge(amount: u32) -> Bucket {
            ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "TestBadge")
                .initial_supply_fungible(amount)
        }

        pub fn combine() -> Bucket {
            let mut bucket1 = Self::create_test_badge(100);
            let bucket2 = bucket1.take(50);
            bucket1.put(bucket2);
            bucket1
        }

        pub fn split() -> (Bucket, Bucket) {
            let bucket1 = Self::create_test_badge(100);
            let bucket2 = bucket1.take(Decimal::from(5));
            (bucket1, bucket2)
        }

        pub fn borrow() -> Bucket {
            let bucket = Self::create_test_badge(100);
            let bucket_ref = bucket.present();
            bucket_ref.drop();
            bucket
        }

        pub fn query() -> (Decimal, Address, Bucket) {
            let bucket = Self::create_test_badge(100);
            (bucket.amount(), bucket.resource_address(), bucket)
        }
    }
}
