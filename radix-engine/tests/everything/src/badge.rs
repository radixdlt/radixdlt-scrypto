use scrypto::blueprint;
use scrypto::resource::{Bucket, ResourceBuilder};
use scrypto::types::{Address, Decimal};

blueprint! {
    struct BadgeTest;

    impl BadgeTest {
        pub fn combine() -> Bucket {
            let bucket1 = ResourceBuilder::new()
                .metadata("name", "TestBadge")
                .new_badge_fixed(100);
            let bucket2 = bucket1.take(50);

            bucket1.put(bucket2);
            bucket1
        }

        pub fn split() -> (Bucket, Bucket) {
            let bucket1 = ResourceBuilder::new()
                .metadata("name", "TestBadge")
                .new_badge_fixed(100);
            let bucket2 = bucket1.take(Decimal::from(5));
            (bucket1, bucket2)
        }

        pub fn borrow() -> Bucket {
            let bucket = ResourceBuilder::new()
                .metadata("name", "TestBadge")
                .new_badge_fixed(100);

            let bucket_ref = bucket.borrow();
            bucket_ref.drop();
            bucket
        }

        pub fn query() -> (Decimal, Address, Bucket) {
            let bucket = ResourceBuilder::new()
                .metadata("name", "TestBadge")
                .new_badge_fixed(100);

            (bucket.amount(), bucket.resource_address(), bucket)
        }
    }
}
