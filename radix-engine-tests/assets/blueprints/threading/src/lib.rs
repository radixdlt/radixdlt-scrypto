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

        pub fn new2(reservation: GlobalAddressReservation) -> ComponentAddress {
            Self {
                vault: Vault::new(XRD),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .with_address(reservation)
            .globalize()
            .address()
        }

        pub fn create_locked_bucket(&mut self, amount: Decimal) -> (Bucket, Proof) {
            let bucket = self.vault.take(amount);
            let proof = bucket.create_proof_of_all();
            (bucket, proof)
        }

        pub fn call(node_id: NodeId) {
            let address = unsafe { ComponentAddress::new_unchecked(node_id.into()) };
            let component: Global<Threading> = address.into();
            component.method();
        }

        pub fn method(&self) {}
    }
}
