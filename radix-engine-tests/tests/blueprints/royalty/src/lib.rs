use scrypto::prelude::*;

#[blueprint]
mod royalty_test {
    struct RoyaltyTest {}

    impl RoyaltyTest {
        pub fn paid_method(&self) -> u32 {
            0
        }

        pub fn paid_method_panic(&self) -> u32 {
            panic!("Boom!")
        }

        pub fn free_method(&self) -> u32 {
            1
        }

        pub fn create_component_with_royalty_enabled() -> Global<RoyaltyTest> {
            let mut local_component = Self {}.instantiate();

            let royalty = {
                let config = RoyaltyConfigBuilder::new()
                    .add_rule("paid_method", 1)
                    .add_rule("paid_method_panic", 1)
                    .default(0);
                Royalty::new(config)
            };

            let access_rules = {
                let mut authority_rules = AuthorityRules::new();
                authority_rules.set_rule("owner", rule!(allow_all), rule!(allow_all));
                AccessRules::new(MethodAuthorities::new(), authority_rules)
            };

            local_component
                .attach_royalty(royalty)
                .attach_access_rules(access_rules)
                .globalize()
        }
    }
}
