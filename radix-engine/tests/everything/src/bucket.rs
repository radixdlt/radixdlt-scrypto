use crate::utils::*;
use scrypto::constructs::*;
use scrypto::resource::*;
use scrypto::types::*;
use scrypto::*;

blueprint! {
    struct BucketTest;

    impl BucketTest {

        pub fn combine() -> Bucket {
            let resource_def = create_mutable("b1", Context::package_address());
            let bucket1 = resource_def.mint(50);
            let bucket2 = resource_def.mint(50);

            bucket1.put(bucket2);
            bucket1
        }

        pub fn split()  -> (Bucket, Bucket) {
            let resource_def = create_mutable("b2", Context::package_address());
            let bucket1 = resource_def.mint(100);
            let bucket2 = bucket1.take(Amount::from(5));
            (bucket1, bucket2)
        }

        pub fn borrow() -> Bucket {
            let resource_def = create_mutable("b3", Context::package_address());
            let bucket = resource_def.mint(100);
            let reference = bucket.borrow();
            reference.drop();
            bucket
        }

        pub fn query() -> (Amount, Address, Bucket) {
            let resource_def = create_mutable("b4", Context::package_address());
            let bucket = resource_def.mint(100);
            (bucket.amount(), bucket.resource_address(), bucket)
        }
    }
}
