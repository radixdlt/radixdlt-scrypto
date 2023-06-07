use scrypto::prelude::*;

#[blueprint]
mod royalty_test {
    enable_method_auth! {
        methods {
            paid_method => PUBLIC;
            paid_method_panic => PUBLIC;
            free_method => PUBLIC;
        },
        royalties {
            claim_royalty => OWNER;
            set_royalty_config => OWNER;
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
                package.set_royalty("RoyaltyTest", "paid_method", RoyaltyAmount::Xrd(2.into()));
                package.set_royalty(
                    "RoyaltyTest",
                    "paid_method_panic",
                    RoyaltyAmount::Xrd(2.into()),
                );
            })
        }

        pub fn create_component_with_royalty_enabled(
            badge: NonFungibleGlobalId,
        ) -> Global<RoyaltyTest> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::Updateable(rule!(require(badge))))
                .royalties(royalties! {
                    paid_method => Xrd(1.into()),
                    paid_method_panic => Xrd(1.into()),
                    free_method => Free,
                })
                .globalize()
        }

        pub fn disable_package_royalty(package: Package, proof: Proof) {
            proof.authorize(|| {
                package.set_royalty("RoyaltyTest", "paid_method", RoyaltyAmount::Free);
                package.set_royalty("RoyaltyTest", "paid_method_panic", RoyaltyAmount::Free);
            })
        }

        pub fn claim_package_royalty(package: Package, proof: Proof) -> Bucket {
            proof.authorize(|| package.claim_royalty())
        }

        pub fn claim_component_royalty(component: Global<AnyComponent>, proof: Proof) -> Bucket {
            proof.authorize(|| {
                let royalty = component.royalty();
                royalty.claim_royalties()
            })
        }
    }
}
