use scrypto::blueprint;
use scrypto::core::Context;
use scrypto::types::*;

blueprint! {
    struct ContextTest;

    impl ContextTest {
        pub fn query() -> (Address, H256, u64) {
            (Context::package_address(), Context::transaction_hash(), Context::current_epoch())
        }
    }
}
