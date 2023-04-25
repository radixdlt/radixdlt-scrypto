use scrypto::prelude::*;

#[blueprint]
mod data_validation {
    struct DataValidation {}

    impl DataValidation {
        pub fn new() -> ComponentAddress {
            Self {}.instantiate().globalize()
        }

        pub fn accept_empty_bucket(&self, bucket: Bucket) {
            bucket.drop_empty()
        }

        pub fn accept_non_empty_bucket(&self, bucket: Bucket) -> Bucket {
            bucket
        }

        pub fn accept_proof(&self, proof: Proof) {
            proof.drop()
        }
    }
}
