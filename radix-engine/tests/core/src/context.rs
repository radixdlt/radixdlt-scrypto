use scrypto::prelude::*;

blueprint! {
    struct CoreTest;

    impl CoreTest {
        pub fn query() -> (PackageAddress, Hash, u64, u128) {
            (
                Runtime::package_address(),
                Runtime::transaction_hash(),
                Runtime::current_epoch(),
                Runtime::generate_uuid(),
            )
        }
    }
}
