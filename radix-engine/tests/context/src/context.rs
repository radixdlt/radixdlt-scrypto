use scrypto::prelude::*;

blueprint! {
    struct ContextTest;

    impl ContextTest {
        pub fn query() -> (PackageId, Hash, u64, u128) {
            (
                Context::package_id(),
                Context::transaction_hash(),
                Context::current_epoch(),
                Context::generate_uuid(),
            )
        }
    }
}
