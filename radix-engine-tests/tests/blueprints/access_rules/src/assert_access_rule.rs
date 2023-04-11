use scrypto::prelude::*;

#[blueprint]
mod assert_access_rule {
    struct AssertAccessRule {}

    impl AssertAccessRule {
        pub fn new() -> ComponentAddress {
            Self {}.instantiate().globalize()
        }

        pub fn assert_access_rule(&self, access_rule: AccessRule) {
            Runtime::assert_access_rule(access_rule);
        }
    }
}
