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
            claim_royalty => Public;
            set_royalty_config => [];
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

        pub fn create_component_with_royalty_enabled() -> Global<RoyaltyTest> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .royalties(royalties! {
                    init {
                        free_method => Free;
                        paid_method => 1u32;
                        paid_method_panic => 1u32;
                    }
                })
                .globalize()
        }
    }
}
