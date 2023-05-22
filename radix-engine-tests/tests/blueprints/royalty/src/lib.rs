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
            Self {}
                .instantiate()
                .prepare_to_globalize()
                .define_roles(roles! {
                    "public" => rule!(allow_all);
                })
                .protect_royalty(protect! {
                    RoyaltyMethod::claim_royalty => ["public"];
                })
                .set_royalties(royalties! {
                    Method::paid_method => 1u32;
                    Method::paid_method_panic => 1u32;
                })
                .globalize()
        }
    }
}
