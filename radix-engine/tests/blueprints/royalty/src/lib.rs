use scrypto::prelude::*;

blueprint! {
    struct RoyaltyTest {}

    impl RoyaltyTest {
        pub fn paid_method(&self) -> u32 {
            0
        }

        pub fn free_method(&self) -> u32 {
            1
        }

        pub fn create_component_with_royalty_enabled() -> ComponentAddress {
            let mut local_component = Self {}.instantiate();

            local_component.set_royalty_config(
                RoyaltyConfigBuilder::new()
                    .add_rule("paid_method", dec!("0.1"))
                    .default(dec!("0")),
            );

            local_component.globalize()
        }

        pub fn enable_royalty() -> PackageAddress {
            let package_address = Runtime::package_address();

            borrow_package!(package_address).set_royalty_config(HashMap::from([(
                "RoyaltyTest".to_owned(),
                RoyaltyConfigBuilder::new()
                    .add_rule("paid_method", dec!("0.2"))
                    .default(dec!("0")),
            )]));

            package_address
        }
    }
}
