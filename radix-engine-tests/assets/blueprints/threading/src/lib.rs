use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct MyData {}

#[blueprint]
mod threading {

    struct Threading {
        vault: Vault,
    }

    impl Threading {
        pub fn new(bucket: Bucket) -> ComponentAddress {
            Self {
                vault: Vault::with_bucket(bucket),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
            .address()
        }

        pub fn create_locked_bucket(&mut self, amount: Decimal) -> (Bucket, Proof) {
            let bucket = self.vault.take(amount);
            let proof = bucket.create_proof_of_all();
            (bucket, proof)
        }
    }
}
