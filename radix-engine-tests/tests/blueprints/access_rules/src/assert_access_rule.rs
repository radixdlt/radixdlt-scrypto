use scrypto::prelude::*;

#[blueprint]
mod assert_access_rule {
    struct AssertAccessRule {}

    impl AssertAccessRule {
        pub fn new() -> ComponentAddress {
            Self {}.instantiate().globalize()
        }

        pub fn assert_access_rule(
            &self,
            access_rule: AccessRule,
            buckets: Vec<Bucket>,
        ) -> Vec<Bucket> {
            for bucket in buckets.iter() {
                ComponentAuthZone::push(bucket.create_proof())
            }

            ComponentAuthZone::assert_access_rule(access_rule);

            for _ in buckets.iter() {
                ComponentAuthZone::pop().drop();
            }

            buckets
        }
    }
}
