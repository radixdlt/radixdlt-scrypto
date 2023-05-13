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

        pub fn create_component_with_royalty_enabled() -> ComponentAddress {
            let local_component = Self {}.instantiate();

            let config = RoyaltyConfigBuilder::new()
                .add_rule("paid_method", 1)
                .add_rule("paid_method_panic", 1)
                .default(0);

            let mut authority_rules = AuthorityRules::new();
            authority_rules.set_rule("owner", rule!(allow_all), rule!(allow_all));

            local_component.globalize_with_modules(
                AccessRules::new(
                    MethodAuthorities::new(),
                    authority_rules,
                ),
                Metadata::new(),
                Royalty::new(config),
            )
        }
    }
}
