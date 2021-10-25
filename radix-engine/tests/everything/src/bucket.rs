use scrypto::blueprint;
use scrypto::resource::Bucket;
use scrypto::types::{Address, Amount};

use crate::utils::*;

blueprint! {
    struct BucketTest;

    impl BucketTest {

        pub fn combine() -> Bucket {
            let (resource_def, auth) = create_mutable("b1");
            let bucket1 = resource_def.mint(50, auth.borrow());
            let bucket2 = resource_def.mint(50, auth.borrow());
            auth.burn();

            bucket1.put(bucket2);
            bucket1
        }

        pub fn split()  -> (Bucket, Bucket) {
            let (resource_def, auth) = create_mutable("b2");
            let bucket1 = resource_def.mint(100, auth.borrow());
            auth.burn();

            let bucket2 = bucket1.take(Amount::from(5));
            (bucket1, bucket2)
        }

        pub fn borrow() -> Bucket {
            let (resource_def, auth) = create_mutable("b3");
            let bucket = resource_def.mint(100, auth.borrow());
            auth.burn();

            let bucket_ref = bucket.borrow();
            bucket_ref.drop();
            bucket
        }

        pub fn query() -> (Amount, Address, Bucket) {
            let (resource_def, auth) = create_mutable("b4");
            let bucket = resource_def.mint(100, auth.borrow());
            auth.burn();

            (bucket.amount(), bucket.resource_def().address(), bucket)
        }
    }
}
