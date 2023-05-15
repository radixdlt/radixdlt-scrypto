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

        pub fn enable_royalty_for_package(package: Package, proof: Proof) {
            proof.authorize(|| {
                package.set_royalty_config(BTreeMap::from([(
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
        ) -> Global<RoyaltyTest> {
            let local_component = Self {}.instantiate();
            let royalty = {
                let royalty_config = RoyaltyConfigBuilder::new()
                    .add_rule("paid_method", 1)
                    .add_rule("paid_method_panic", 1)
                    .default(0);
                Royalty::new(royalty_config)
            };

            let access_rules = {
                let mut authority_rules = AuthorityRules::new();
                authority_rules.set_rule(
                    "owner".clone(),
                    rule!(require(badge.clone())),
                    rule!(require(badge.clone())),
                );
                AccessRules::new(MethodAuthorities::new(), authority_rules)
            };

            local_component
                .attach_royalty(royalty)
                .attach_access_rules(access_rules)
                .globalize()
        }

        pub fn disable_package_royalty(package: Package, proof: Proof) {
            proof.authorize(|| {
                package.set_royalty_config(BTreeMap::from([]));
            })
        }

        pub fn disable_component_royalty(address: ComponentAddress, proof: Proof) {
            proof.authorize(|| {
                let component: Global<AnyComponent> = address.into();
                let royalty = component.royalty();
                royalty.set_config(RoyaltyConfig::default());
            })
        }

        pub fn claim_package_royalty(package: Package, proof: Proof) -> Bucket {
            proof.authorize(|| package.claim_royalty())
        }

        pub fn claim_component_royalty(address: ComponentAddress, proof: Proof) -> Bucket {
            proof.authorize(|| {
                let component: Global<AnyComponent> = address.into();
                let royalty = component.royalty();
                royalty.claim_royalty()
            })
        }
    }
}
