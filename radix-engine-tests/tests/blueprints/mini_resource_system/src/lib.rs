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

        pub fn drop(proof: MiniProofComponent) {
            ScryptoEnv
                .drop_object(proof.component.as_node_id())
                .unwrap();
        }
    }
}

#[blueprint]
mod mini_bucket {
    use crate::mini_proof::MiniProofComponent;

    struct MiniBucket {
        amount: u32,
    }

    impl MiniBucket {
        pub fn new(amount: u32) -> MiniBucketComponent {
            Self { amount }.instantiate().own()
        }

        pub fn amount(&self) -> u32 {
            self.amount
        }

        pub fn create_proof(&self) -> MiniProofComponent {
            let node_id = Runtime::node_id();
            crate::mini_proof::MiniProof {
                bucket: InternalAddress::new_or_panic(node_id.into()),
            }
            .instantiate()
            .own()
        }

        pub fn drop(bucket: MiniBucketComponent) {
            ScryptoEnv
                .drop_object(bucket.component.as_node_id())
                .unwrap();
        }
    }
}

#[blueprint]
mod mini_user {
    use crate::mini_bucket::MiniBucketComponent;
    use crate::mini_proof::MiniProofComponent;

    struct MiniUser {}

    impl MiniUser {
        // Case 1
        pub fn create_bucket_proof_and_do_nothing() {
            let bucket = MiniBucketComponent::new(5);
            let _proof = bucket.create_proof();
        }

        // Case 2
        pub fn create_bucket_proof_and_query_amount() {
            let bucket = MiniBucketComponent::new(5);
            let proof = bucket.create_proof();
            assert_eq!(proof.amount(), 5);
            MiniProofComponent::drop(proof);
            MiniBucketComponent::drop(bucket);
        }

        // Case 3
        pub fn create_bucket_proof_and_drop_proof_and_drop_bucket() {
            let bucket = MiniBucketComponent::new(5);
            let proof = bucket.create_proof();
            MiniProofComponent::drop(proof);
            MiniBucketComponent::drop(bucket);
        }

        // Case 4
        pub fn create_bucket_proof_and_drop_bucket_and_drop_proof() {
            let bucket = MiniBucketComponent::new(5);
            let proof = bucket.create_proof();
            MiniBucketComponent::drop(bucket);
            MiniProofComponent::drop(proof);
        }

        // Case 5
        pub fn create_bucket_proof_and_return_both() {
            let (bucket, proof) = MiniUserComponent::create_bucket_and_proof();
            MiniProofComponent::drop(proof);
            MiniBucketComponent::drop(bucket);
        }

        pub fn create_bucket_and_proof() -> (MiniBucketComponent, MiniProofComponent) {
            let bucket = MiniBucketComponent::new(5);
            let proof = bucket.create_proof();
            (bucket, proof)
        }

        // Case 6
        pub fn create_proof_and_drop_the_bucket_in_another_frame() {
            let bucket = MiniBucketComponent::new(5);
            let proof = bucket.create_proof();
            MiniUserComponent::drop_bucket(bucket);
            MiniProofComponent::drop(proof);
        }

        pub fn drop_bucket(bucket: MiniBucketComponent) {
            MiniBucketComponent::drop(bucket);
        }

        // Case 7
        pub fn create_proof_and_drop_the_proof_in_another_frame() {
            let bucket = MiniBucketComponent::new(5);
            let proof = bucket.create_proof();
            MiniUserComponent::drop_proof(proof);
            MiniBucketComponent::drop(bucket);
        }

        pub fn drop_proof(proof: MiniProofComponent) {
            MiniProofComponent::drop(proof);
        }
    }
}
