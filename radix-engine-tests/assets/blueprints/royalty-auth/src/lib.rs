use scrypto::prelude::*;

#[blueprint]
mod royalty_test {
    enable_package_royalties! {
        paid_method => Xrd(2.into());
        paid_method_panic => Xrd(2.into());
        free_method => Free;
        create_component_with_royalty_enabled => Free;
        claim_package_royalty => Free;
        claim_component_royalty => Free;
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

        pub fn create_component_with_royalty_enabled(
            badge: NonFungibleGlobalId,
        ) -> Global<RoyaltyTest> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::Updatable(rule!(require(badge.clone()))))
                .enable_component_royalties(component_royalties! {
                    init {
                        paid_method => Xrd(1.into()), updatable;
                        paid_method_panic => Xrd(1.into()), locked;
                        free_method => Free, locked;
                    }
                })
                .globalize()
        }

        pub fn claim_package_royalty(package: Package, proof: Proof) -> FungibleBucket {
            proof.authorize(|| package.claim_royalties())
        }

        pub fn claim_component_royalty(
            component: Global<AnyComponent>,
            proof: Proof,
        ) -> FungibleBucket {
            proof.authorize(|| component.claim_component_royalties())
        }
    }
}
