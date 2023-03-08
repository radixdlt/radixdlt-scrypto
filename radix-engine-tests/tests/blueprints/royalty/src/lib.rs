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

            local_component.globalize_with_royalty_config(config)
        }

        pub fn enable_royalty_for_this_package() {
            let package_address = Runtime::package_address();

            borrow_package!(package_address).set_royalty_config(BTreeMap::from([(
                "RoyaltyTest".to_owned(),
                RoyaltyConfigBuilder::new()
                    .add_rule("paid_method", 2)
                    .add_rule("paid_method_panic", 2)
                    .default(0),
            )]));
        }

        pub fn claim_package_royalty(address: PackageAddress) -> Bucket {
            borrow_package!(address).claim_royalty()
        }

        pub fn claim_component_royalty(address: ComponentAddress) -> Bucket {
            let royalty = borrow_component!(address).royalty();
            royalty.claim_royalty()
        }
    }
}
