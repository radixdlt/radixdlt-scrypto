use scrypto::prelude::*;

blueprint! {
    struct CoreTest;

    impl CoreTest {
        pub fn query() -> (PackageId, Hash, u64, u128) {
            (
                Process::package_id(),
                Transaction::transaction_hash(),
                Transaction::current_epoch(),
                Process::generate_uuid(),
            )
        }
    }
}
