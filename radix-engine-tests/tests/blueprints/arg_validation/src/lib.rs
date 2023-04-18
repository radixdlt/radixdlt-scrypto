use scrypto::prelude::*;

#[blueprint]
mod abi_validation {
    struct ArgValidation {}

    impl ArgValidation {
        pub fn new() -> ComponentAddress {
            Self {}.instantiate().globalize()
        }

        pub fn accept_empty_bucket(&self, bucket: Bucket) {
            bucket.drop_empty()
        }

        pub fn accept_and_return_bucket(&self, bucket: Bucket) -> Bucket {
            bucket
        }

        pub fn accept_proof(&self, _proof: Proof) {}
    }
}
