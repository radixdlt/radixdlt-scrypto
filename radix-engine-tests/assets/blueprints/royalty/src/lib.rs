use scrypto::prelude::*;

#[blueprint]
mod royalty_test {
    enable_package_royalties! {
        paid_method => Xrd(2.into());
        paid_method_usd => Free;
        paid_method_panic => Xrd(2.into());
        free_method => Free;
        create_component_with_royalty_enabled => Free;
        create_component_with_royalty => Free;
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
                .enable_component_royalties(component_royalties! {
                    roles {
                        royalty_setter => rule!(allow_all);
                        royalty_setter_updater => rule!(deny_all);
                        royalty_locker => rule!(allow_all);
                        royalty_locker_updater => rule!(deny_all);
                        royalty_claimer => rule!(allow_all);
                        royalty_claimer_updater => rule!(deny_all);
                    },
                    init {
                        free_method => Free, updatable;
                        paid_method => Xrd(1.into()), updatable;
                        paid_method_usd => Usd(1.into()), updatable;
                        paid_method_panic => Xrd(1.into()), updatable;
                    }
                })
                .globalize()
        }

        pub fn create_component_with_royalty(amount: Decimal) -> Global<RoyaltyTest> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .enable_component_royalties(component_royalties! {
                    roles {
                        royalty_setter => rule!(allow_all);
                        royalty_setter_updater => rule!(deny_all);
                        royalty_locker => rule!(allow_all);
                        royalty_locker_updater => rule!(deny_all);
                        royalty_claimer => rule!(allow_all);
                        royalty_claimer_updater => rule!(deny_all);
                    },
                    init {
                        free_method => Free, updatable;
                        paid_method => Xrd(amount), updatable;
                        paid_method_usd => Usd(1.into()), updatable;
                        paid_method_panic => Xrd(1.into()), updatable;
                    }
                })
                .globalize()
        }
    }
}
