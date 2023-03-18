use scrypto::prelude::*;

#[blueprint]
mod transaction_limits {
    struct TransactionRuntimeTest {}

    impl TransactionRuntimeTest {
        pub fn get_transaction_hash() -> Hash {
            Runtime::transaction_hash()
        }

        pub fn generate_uuid() -> u128 {
            Runtime::generate_uuid()
        }
    }
}
