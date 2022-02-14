use scrypto::prelude::*;

blueprint! {
    struct ContextTest;

    impl ContextTest {
        pub fn query() -> (Actor, PackageRef, Hash, u64, u128) {
            (
                Context::actor(),
                Context::package(),
                Context::transaction_hash(),
                Context::current_epoch(),
                Context::generate_uuid(),
            )
        }
    }
}
