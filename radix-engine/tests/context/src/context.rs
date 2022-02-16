use scrypto::prelude::*;

blueprint! {
    struct ContextTest;

    impl ContextTest {
        pub fn query() -> (PackageRef, Hash, u64, u128) {
            (
                Context::package_ref(),
                Context::transaction_hash(),
                Context::current_epoch(),
                Context::generate_uuid(),
            )
        }
    }
}
