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
            let bucket = Blueprint::<MiniBucket>::new(5);
            let _proof = bucket.create_proof();
        }

        // Case 2
        pub fn create_bucket_proof_and_query_amount() {
            let bucket = Blueprint::<MiniBucket>::new(5);
            let proof = bucket.create_proof();
            assert_eq!(proof.amount(), 5);
            Blueprint::<MiniProof>::drop(proof);
            Blueprint::<MiniBucket>::drop(bucket);
        }

        // Case 3
        pub fn create_bucket_proof_and_drop_proof_and_drop_bucket() {
            let bucket = Blueprint::<MiniBucket>::new(5);
            let proof = bucket.create_proof();
            Blueprint::<MiniProof>::drop(proof);
            Blueprint::<MiniBucket>::drop(bucket);
        }

        // Case 4
        pub fn create_bucket_proof_and_drop_bucket_and_drop_proof() {
            let bucket = Blueprint::<MiniBucket>::new(5);
            let proof = bucket.create_proof();
            Blueprint::<MiniBucket>::drop(bucket);
            Blueprint::<MiniProof>::drop(proof);
        }

        // Case 5
        pub fn create_bucket_proof_and_return_both() {
            let (bucket, proof) = Blueprint::<MiniUser>::create_bucket_and_proof();
            Blueprint::<MiniProof>::drop(proof);
            Blueprint::<MiniBucket>::drop(bucket);
        }

        pub fn create_bucket_and_proof() -> (Owned<MiniBucket>, Owned<MiniProof>) {
            let bucket = Blueprint::<MiniBucket>::new(5);
            let proof = bucket.create_proof();
            (bucket, proof)
        }

        // Case 6
        pub fn create_proof_and_drop_the_bucket_in_another_frame() {
            let bucket = Blueprint::<MiniBucket>::new(5);
            let proof = bucket.create_proof();
            Blueprint::<MiniUser>::drop_bucket(bucket);
            Blueprint::<MiniProof>::drop(proof);
        }

        pub fn drop_bucket(bucket: Owned<MiniBucket>) {
            Blueprint::<MiniBucket>::drop(bucket);
        }

        // Case 7
        pub fn create_proof_and_drop_the_proof_in_another_frame() {
            let bucket = Blueprint::<MiniBucket>::new(5);
            let proof = bucket.create_proof();
            Blueprint::<MiniUser>::drop_proof(proof);
            Blueprint::<MiniBucket>::drop(bucket);
        }

        pub fn drop_proof(proof: Owned<MiniProof>) {
            Blueprint::<MiniProof>::drop(proof);
        }
    }
}
