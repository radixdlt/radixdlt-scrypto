use scrypto::prelude::*;

blueprint! {
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

        // Doesn't really work because of proof downstream movement limitation
        // TODO: make it work by changing the rule to "1-barrier".

        pub fn enable_royalty_for_package(address: PackageAddress, proof: Proof) {
            proof.authorize(|| {
                borrow_package!(address).set_royalty_config(HashMap::from([(
                    "RoyaltyTest".to_owned(),
                    RoyaltyConfigBuilder::new()
                        .add_rule("paid_method", dec!("0.2"))
                        .add_rule("paid_method_panic", dec!("0.2"))
                        .default(dec!("0")),
                )]));
            })
        }

        pub fn create_component_with_royalty_enabled(
            badge: NonFungibleAddress,
        ) -> ComponentAddress {
            let mut local_component = Self {}.instantiate();

            local_component.set_royalty_config(
                RoyaltyConfigBuilder::new()
                    .add_rule("paid_method", dec!("0.1"))
                    .add_rule("paid_method_panic", dec!("0.1"))
                    .default(dec!("0")),
            );
            local_component.set_royalty_owner(badge);

            local_component.globalize_no_owner()
        }

        pub fn disable_package_royalty(address: PackageAddress, proof: Proof) {
            proof.authorize(|| {
                borrow_package!(address).set_royalty_config(HashMap::from([]));
            })
        }

        pub fn disable_component_royalty(address: ComponentAddress, proof: Proof) {
            proof.authorize(|| {
                borrow_component!(address).set_royalty_config(RoyaltyConfig::default());
            })
        }

        pub fn claim_package_royalty(address: PackageAddress, proof: Proof) -> Bucket {
            proof.authorize(|| borrow_package!(address).claim_royalty())
        }

        pub fn claim_component_royalty(address: ComponentAddress, proof: Proof) -> Bucket {
            proof.authorize(|| borrow_component!(address).claim_royalty())
        }
    }
}
