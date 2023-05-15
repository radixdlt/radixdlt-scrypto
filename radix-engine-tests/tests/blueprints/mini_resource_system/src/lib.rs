use scrypto::api::*;
use scrypto::engine::scrypto_env::ScryptoEnv;
use scrypto::prelude::*;

// TODO: need to update XyzComponent schema type!!!

#[blueprint]
mod mini_proof {
    struct MiniProof {
        pub bucket: InternalAddress,
    }

    impl MiniProof {
        pub fn amount(&self) -> u32 {
            scrypto_decode(
                &ScryptoEnv
                    .call_method(self.bucket.as_node_id(), "amount", scrypto_args!())
                    .unwrap(),
            )
            .unwrap()
        }

        pub fn drop(proof: Owned<MiniProof>) {
            ScryptoEnv
                .drop_object(proof.0.handle().as_node_id())
                .unwrap();
        }
    }
}

#[blueprint]
mod mini_bucket {
    use crate::mini_proof::*;

    struct MiniBucket {
        amount: u32,
    }

    impl MiniBucket {
        pub fn new(amount: u32) -> Owned<MiniBucket> {
            Self { amount }.instantiate()
        }

        pub fn amount(&self) -> u32 {
            self.amount
        }

        pub fn create_proof(&self) -> Owned<MiniProof> {
            let node_id = Runtime::node_id();
            crate::mini_proof::MiniProof {
                bucket: InternalAddress::new_or_panic(node_id.into()),
            }
            .instantiate()
        }

        pub fn drop(bucket: Owned<MiniBucket>) {
            ScryptoEnv
                .drop_object(bucket.0.handle().as_node_id())
                .unwrap();
        }
    }
}

#[blueprint]
mod mini_user {
    use crate::mini_bucket::*;
    use crate::mini_proof::*;

    struct MiniUser {}

    impl MiniUser {
        // Case 1
        pub fn create_bucket_proof_and_do_nothing() {
            let bucket = MiniBucketObjectStub::new(5);
            let _proof = bucket.create_proof();
        }

        // Case 2
        pub fn create_bucket_proof_and_query_amount() {
            let bucket = MiniBucketObjectStub::new(5);
            let proof = bucket.create_proof();
            assert_eq!(proof.amount(), 5);
            MiniProofObjectStub::drop(proof);
            MiniBucketObjectStub::drop(bucket);
        }

        // Case 3
        pub fn create_bucket_proof_and_drop_proof_and_drop_bucket() {
            let bucket = MiniBucketObjectStub::new(5);
            let proof = bucket.create_proof();
            MiniProofObjectStub::drop(proof);
            MiniBucketObjectStub::drop(bucket);
        }

        // Case 4
        pub fn create_bucket_proof_and_drop_bucket_and_drop_proof() {
            let bucket = MiniBucketObjectStub::new(5);
            let proof = bucket.create_proof();
            MiniBucketObjectStub::drop(bucket);
            MiniProofObjectStub::drop(proof);
        }

        // Case 5
        pub fn create_bucket_proof_and_return_both() {
            let (bucket, proof) = MiniUserObjectStub::create_bucket_and_proof();
            MiniProofObjectStub::drop(proof);
            MiniBucketObjectStub::drop(bucket);
        }

        pub fn create_bucket_and_proof() -> (Owned<MiniBucket>, Owned<MiniProof>) {
            let bucket = MiniBucketObjectStub::new(5);
            let proof = bucket.create_proof();
            (bucket, proof)
        }

        // Case 6
        pub fn create_proof_and_drop_the_bucket_in_another_frame() {
            let bucket = MiniBucketObjectStub::new(5);
            let proof = bucket.create_proof();
            MiniUserObjectStub::drop_bucket(bucket);
            MiniProofObjectStub::drop(proof);
        }

        pub fn drop_bucket(bucket: Owned<MiniBucket>) {
            MiniBucketObjectStub::drop(bucket);
        }

        // Case 7
        pub fn create_proof_and_drop_the_proof_in_another_frame() {
            let bucket = MiniBucketObjectStub::new(5);
            let proof = bucket.create_proof();
            MiniUserObjectStub::drop_proof(proof);
            MiniBucketObjectStub::drop(bucket);
        }

        pub fn drop_proof(proof: Owned<MiniProof>) {
            MiniProofObjectStub::drop(proof);
        }
    }
}
