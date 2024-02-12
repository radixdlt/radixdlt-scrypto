use scrypto::prelude::*;

#[blueprint]
mod royalties {
    enable_package_royalties! {
        new => Free;
        call_no_package_royalty => Free;
        call_xrd_package_royalty => Xrd(31.into());
        call_usd_package_royalty => Usd(1.into());
    }

    pub struct RoyaltiesBp {
    }

    impl RoyaltiesBp {
        pub fn new() -> Global<RoyaltiesBp> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .enable_component_royalties(component_royalties! {
                    roles {
                        royalty_setter => rule!(allow_all);
                        royalty_setter_updater => rule!(allow_all);
                        royalty_locker => rule!(allow_all);
                        royalty_locker_updater => rule!(allow_all);
                        royalty_claimer => rule!(allow_all);
                        royalty_claimer_updater => rule!(allow_all);
                    },
                    init {
                        // The values below are irrelevant (needed only to enable the component royalties).
                        // The `royalties` Scenario sets these amounts explicitly (so that all package+component combinations are covered).
                        call_no_package_royalty => Usd(6.into()), updatable;
                        call_xrd_package_royalty => Free, updatable;
                        call_usd_package_royalty => Xrd(16.into()), updatable;
                    }
                })
                .globalize()
        }

        pub fn call_no_package_royalty(&self) {
        }

        pub fn call_xrd_package_royalty(&self) {
        }

        pub fn call_usd_package_royalty(&self) {
        }
    }
}
