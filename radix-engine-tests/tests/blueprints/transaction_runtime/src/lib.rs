use scrypto::prelude::*;

#[blueprint]
mod tx_runtime {
    struct TransactionRuntimeTest {}

    impl TransactionRuntimeTest {
        pub fn query() -> (PackageAddress, Hash, Epoch) {
            (
                Runtime::package_address(),
                Runtime::transaction_hash(),
                Runtime::current_epoch(),
            )
        }
        pub fn generate_ruid() -> [u8; 32] {
            Runtime::generate_ruid()
        }
    }
}
