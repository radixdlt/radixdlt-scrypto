use scrypto::prelude::*;

blueprint! {
    struct ContextTest;

    impl ContextTest {
        pub fn query() -> (Actor, Address, H256, u64, u128) {
            (
                Context::actor(),
                Context::package_address(),
                Context::transaction_hash(),
                Context::current_epoch(),
                Uuid::generate(),
            )
        }
    }
}
