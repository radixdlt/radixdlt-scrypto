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

        // Doesn't really work because of proof downstream movement limitation
        // TODO: make it work by changing the rule to "1-barrier".

        pub fn enable_royalty_for_package(address: PackageAddress, proof: Proof) {
            proof.authorize(|| {
                borrow_package!(address).set_royalty_config(BTreeMap::from([(
                    "RoyaltyTest".to_owned(),
                    RoyaltyConfigBuilder::new()
                        .add_rule("paid_method", 2)
                        .add_rule("paid_method_panic", 2)
                        .default(0),
                )]));
            })
        }

        pub fn create_component_with_royalty_enabled(
            badge: NonFungibleGlobalId,
        ) -> ComponentAddress {
            let local_component = Self {}.instantiate();
            let royalty_config = RoyaltyConfigBuilder::new()
                .add_rule("paid_method", 1)
                .add_rule("paid_method_panic", 1)
                .default(0);
            local_component.globalize_with_owner_badge(badge, royalty_config)
        }

        pub fn disable_package_royalty(address: PackageAddress, proof: Proof) {
            proof.authorize(|| {
                borrow_package!(address).set_royalty_config(BTreeMap::from([]));
            })
        }

        pub fn disable_component_royalty(address: ComponentAddress, proof: Proof) {
            proof.authorize(|| {
                let royalty = borrow_component!(address).royalty();
                royalty.set_config(RoyaltyConfig::default());
            })
        }

        pub fn claim_package_royalty(address: PackageAddress, proof: Proof) -> Bucket {
            proof.authorize(|| borrow_package!(address).claim_royalty())
        }

        pub fn claim_component_royalty(address: ComponentAddress, proof: Proof) -> Bucket {
            proof.authorize(|| {
                let royalty = borrow_component!(address).royalty();
                royalty.claim_royalty()
            })
        }
    }
}
