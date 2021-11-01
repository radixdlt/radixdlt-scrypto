use scrypto::blueprint;
use scrypto::resource::{Bucket, ResourceBuilder};
use scrypto::types::{Address, Amount};

blueprint! {
    struct BucketTest;

    impl BucketTest {

        pub fn combine() -> Bucket {
            let bucket1 = ResourceBuilder::new()
                .metadata("name", "TestToken")
                .create_fixed(100);
            let bucket2 = bucket1.take(50);

            bucket1.put(bucket2);
            bucket1
        }

        pub fn split()  -> (Bucket, Bucket) {
            let bucket1 = ResourceBuilder::new()
                .metadata("name", "TestToken")
                .create_fixed(100);
            let bucket2 = bucket1.take(Amount::from(5));
            (bucket1, bucket2)
        }

        pub fn borrow() -> Bucket {
            let bucket = ResourceBuilder::new()
                .metadata("name", "TestToken")
                .create_fixed(100);

            let bucket_ref = bucket.borrow();
            bucket_ref.drop();
            bucket
        }

        pub fn query() -> (Amount, Address, Bucket) {
            let bucket = ResourceBuilder::new()
                .metadata("name", "TestToken")
                .create_fixed(100);

            (bucket.amount(), bucket.resource_def().address(), bucket)
        }
    }
}
