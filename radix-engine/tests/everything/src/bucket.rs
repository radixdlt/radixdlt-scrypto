use crate::utils::*;
use scrypto::constructs::*;
use scrypto::resource::*;
use scrypto::types::*;
use scrypto::*;

blueprint! {
    struct BucketTest;

    impl BucketTest {

        pub fn combine() -> Bucket {
            let resource = create_mutable("b1", Context::package_address());
            let bucket1 = mint_resource(resource, 50);
            let bucket2 = mint_resource(resource, 50);

            bucket1.put(bucket2);
            bucket1
        }

        pub fn split()  -> (Bucket, Bucket) {
            let resource = create_mutable("b2", Context::package_address());
            let bucket1 = mint_resource(resource, 100);
            let bucket2 = bucket1.take(U256::from(5));
            (bucket1, bucket2)
        }

        pub fn borrow() -> Bucket {
            let resource = create_mutable("b3", Context::package_address());
            let bucket = mint_resource(resource, 100);
            let reference = bucket.borrow();
            reference.drop();
            bucket
        }

        pub fn query() -> (U256, Address, Bucket) {
            let resource = create_mutable("b4", Context::package_address());
            let bucket = mint_resource(resource, 100);
            (bucket.amount(), bucket.resource(), bucket)
        }
    }
}
