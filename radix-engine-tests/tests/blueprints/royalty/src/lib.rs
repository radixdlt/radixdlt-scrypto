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
                .royalty("paid_method", 1)
                .royalty("paid_method_panic", 1)
                .royalty_default(0)
                .owner_authority(rule!(allow_all), rule!(allow_all))
                .globalize()
        }
    }
}
