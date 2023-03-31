use scrypto::prelude::*;

#[blueprint]
mod badge_test {
    struct BadgeTest;

    impl BadgeTest {
        fn create_test_badge(amount: u32) -> Bucket {
            ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .restrict_withdraw(rule!(allow_all), rule!(deny_all))
                .metadata("name", "TestBadge")
                .mint_initial_supply(amount)
        }

        pub fn combine() -> Bucket {
            let mut bucket1 = Self::create_test_badge(100);
            let bucket2 = bucket1.take(50);
            bucket1.put(bucket2);
            bucket1
        }

        pub fn split() -> (Bucket, Bucket) {
            let mut bucket1 = Self::create_test_badge(100);
            let bucket2 = bucket1.take(Decimal::from(5));
            (bucket1, bucket2)
        }

        pub fn borrow() -> Bucket {
            let bucket = Self::create_test_badge(100);
            let proof = bucket.create_proof();
            proof.drop();
            bucket
        }

        pub fn query() -> (Decimal, ResourceAddress, Bucket) {
            let bucket = Self::create_test_badge(100);
            (bucket.amount(), bucket.resource_address(), bucket)
        }
    }
}
