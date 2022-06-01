use scrypto::prelude::*;

blueprint! {
    struct AbiComponent {}

    impl AbiComponent {
        pub fn create_component() -> ComponentAddress {
            Self {}.instantiate().globalize()
        }

        pub fn create_invalid_abi_component() -> ComponentAddress {
            Self {}
                .instantiate()
                .add_access_check(
                    AccessRules::new()
                        .method("no_method", rule!(require("something")))
                        .default(rule!(allow_all)),
                )
                .globalize()
        }
    }
}
