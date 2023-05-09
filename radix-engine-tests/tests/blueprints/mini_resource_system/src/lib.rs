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
                .drop_object(proof.component.0.as_node_id())
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
            Self { amount }.instantiate()
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
        }

        pub fn drop(bucket: MiniBucketComponent) {
            ScryptoEnv
                .drop_object(bucket.component.0.as_node_id())
                .unwrap();
        }
    }
}
