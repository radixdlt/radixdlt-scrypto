use scrypto::prelude::*;

blueprint! {
    struct BucketProof;

    impl BucketProof {
        pub fn create_clone_drop_bucket_proof(bucket: Bucket, amount: Decimal) -> Bucket {
            let proof = bucket.create_proof();
            let clone = proof.clone();

            assert_eq!(bucket.amount(), amount);
            assert_eq!(proof.amount(), amount);
            assert_eq!(clone.amount(), amount);

            clone.drop();
            proof.drop();
            bucket
        }

        pub fn create_clone_drop_bucket_proof_by_amount(
            bucket: Bucket,
            total_amount: Decimal,
            proof_amount: Decimal,
        ) -> Bucket {
            let proof = bucket.create_proof_by_amount(proof_amount);
            let clone = proof.clone();

            assert_eq!(bucket.amount(), total_amount);
            assert_eq!(proof.amount(), proof_amount);
            assert_eq!(clone.amount(), proof_amount);

            clone.drop();
            proof.drop();
            bucket
        }

        pub fn create_clone_drop_bucket_proof_by_ids(
            bucket: Bucket,
            total_ids: BTreeSet<NonFungibleId>,
            proof_ids: BTreeSet<NonFungibleId>,
        ) -> Bucket {
            let proof = bucket.create_proof_by_ids(&proof_ids);
            let clone = proof.clone();

            assert_eq!(bucket.get_non_fungible_ids(), total_ids);
            assert_eq!(proof.get_non_fungible_ids(), proof_ids);
            assert_eq!(clone.get_non_fungible_ids(), proof_ids);

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
    }
}
