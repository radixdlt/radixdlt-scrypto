use scrypto::prelude::*;

#[blueprint]
mod bucket_proof {
    struct BucketProof;

    impl BucketProof {
        pub fn create_clone_drop_bucket_proof(bucket: Bucket, amount: Decimal) -> Bucket {
            let proof = bucket.create_proof();
            let proof = proof.validate_proof(bucket.resource_address()).unwrap();
            let clone = proof.clone();

            assert_eq!(bucket.amount(), amount);
            assert_eq!(proof.amount(), amount);
            assert_eq!(clone.amount(), amount);

            clone.drop();
            proof.drop();
            bucket
        }

        pub fn use_bucket_proof_for_auth(bucket: Bucket, to_burn: Bucket) -> Bucket {
            bucket.authorize(|| {
                to_burn.burn();
            });

            bucket
        }

        pub fn return_bucket_while_locked(bucket: Bucket) -> Bucket {
            let _proof = bucket.create_proof();
            bucket
        }
    }
}
