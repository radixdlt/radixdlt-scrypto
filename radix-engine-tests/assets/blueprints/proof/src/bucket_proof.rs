use scrypto::prelude::*;

#[blueprint]
mod bucket_proof {
    struct BucketProof;

    impl BucketProof {
        pub fn create_clone_drop_bucket_proof(bucket: Bucket, amount: Decimal) -> Bucket {
            let proof = bucket.create_proof_of_all().skip_checking();
            assert_eq!(proof.resource_address(), bucket.resource_address());
            let clone = proof.clone();

            assert_eq!(bucket.amount(), amount);
            assert_eq!(proof.amount(), amount);
            assert_eq!(clone.amount(), amount);

            clone.drop();
            proof.drop();
            bucket
        }

        pub fn use_bucket_proof_for_auth(bucket: Bucket, to_burn: Bucket) -> Bucket {
            bucket.authorize_with_all(|| {
                to_burn.burn();
            });

            bucket
        }

        pub fn return_bucket_while_locked(bucket: Bucket) -> Bucket {
            let _proof = bucket.create_proof_of_all();
            bucket
        }

        pub fn check_balance_and_bounce(bucket: Bucket, balance: Decimal) -> Bucket {
            assert_eq!(bucket.as_fungible().amount(), balance);
            bucket
        }

        pub fn split_bucket(mut bucket: Bucket, balance: Decimal) -> (Bucket, Bucket) {
            let taken = bucket.take(balance);
            (bucket, taken)
        }

        pub fn check_proof_amount_and_drop(proof: Proof, balance: Decimal) {
            assert_eq!(proof.skip_checking().amount(), balance);
        }
    }
}
