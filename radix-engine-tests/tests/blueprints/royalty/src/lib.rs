use scrypto::prelude::*;

#[blueprint]
mod royalty_test {
    enable_package_royalties! {
        paid_method => Xrd(2.into()),
        paid_method_usd => Free,
        paid_method_panic => Xrd(2.into()),
        free_method => Free,
        create_component_with_royalty_enabled => Free,
    }

    enable_method_auth! {
        methods {
            paid_method => PUBLIC;
            paid_method_usd => PUBLIC;
            paid_method_panic => PUBLIC;
            free_method => PUBLIC;
        }
    }

    struct RoyaltyTest {}

    impl RoyaltyTest {
        pub fn paid_method(&self) -> u32 {
            0
        }

        pub fn paid_method_usd(&self) -> u32 {
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
                    roles {
                        owner => rule!(allow_all);
                    },
                    init {
                        free_method => Free,
                        paid_method => Xrd(1.into()),
                        paid_method_usd => Usd(1.into()),
                        paid_method_panic => Xrd(1.into()),
                    }
                })
                .globalize()
        }
    }
}
