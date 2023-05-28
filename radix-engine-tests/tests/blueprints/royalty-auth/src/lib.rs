use scrypto::prelude::*;

#[blueprint]
mod royalty_test {
    define_permissions! {
        main {
            paid_method => Public;
            paid_method_panic => Public;
            free_method => Public;
        },
        royalties {
            claim_royalty => [OWNER_ROLE];
            set_royalty_config => [OWNER_ROLE];
        }
    }

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
                package.set_royalty_config(BTreeMap::from([("RoyaltyTest".to_owned(), {
                    let mut config = RoyaltyConfig::default();
                    config.set_rule("paid_method", 2);
                    config.set_rule("paid_method_panic", 2);
                    config
                })]));
            })
        }

        pub fn create_component_with_royalty_enabled(
            badge: NonFungibleGlobalId,
        ) -> Global<RoyaltyTest> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::Updateable(rule!(require(badge))))
                .royalties(royalties!(
                    init {
                        paid_method => 1u32;
                        paid_method_panic => 1u32;
                        free_method => Free;
                    }
                ))
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
