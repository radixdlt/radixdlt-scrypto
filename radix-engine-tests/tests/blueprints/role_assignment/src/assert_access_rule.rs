use scrypto::prelude::*;

#[blueprint]
mod assert_access_rule {
    struct AssertRule {}

    impl AssertRule {
        pub fn new() -> Global<AssertRule> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn assert_access_rule(&self, access_rule: Rule) {
            Runtime::assert_access_rule(access_rule);
        }
    }
}
