use scrypto::prelude::*;

#[blueprint]
mod big_fi {
    use crate::subservio::Subservio;
    use crate::subservio::SubservioFunctions;

    struct BigFi {
        child: Owned<Subservio>,
    }

    impl BigFi {
        pub fn create() -> (Global<BigFi>, NonFungibleBucket) {
            let big_fi_badge = ResourceBuilder::new_integer_non_fungible(OwnerRole::None)
                .mint_initial_supply(vec![(0u64.into(), ())]);

            let big_fi_resource = big_fi_badge.resource_address();

            let child = Blueprint::<Subservio>::create();

            let global = Self { child }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .metadata(metadata! {
                    roles {
                        metadata_locker => rule!(deny_all);
                        metadata_locker_updater => rule!(deny_all);
                        metadata_setter => rule!(require(big_fi_resource));
                        metadata_setter_updater => rule!(deny_all);
                    }
                })
                .globalize();

            (global, big_fi_badge)
        }
    }
}

#[blueprint]
mod subservio {
    struct Subservio {}

    impl Subservio {
        pub fn create() -> Owned<Subservio> {
            Self {}.instantiate()
        }
    }
}

#[blueprint]
mod swappy {
    enable_method_auth! {
        roles {
            some_role => updatable_by: [];
        },
        methods {
            protected_method => restrict_to: [some_role];
        }
    }

    struct Swappy {
        access_rule: AccessRule,
    }

    impl Swappy {
        pub fn create() -> (Global<Swappy>, NonFungibleBucket) {
            let swappy_badge = ResourceBuilder::new_integer_non_fungible(OwnerRole::None)
                .mint_initial_supply(vec![(0u64.into(), ())]);

            let swappy_resource = swappy_badge.resource_address();

            let global = Self {
                access_rule: rule!(require(swappy_resource)),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .roles(roles! {
                some_role => rule!(require(swappy_resource));
            })
            .metadata(metadata! {
                roles {
                    metadata_locker => rule!(deny_all);
                    metadata_locker_updater => rule!(deny_all);
                    metadata_setter => rule!(require(swappy_resource));
                    metadata_setter_updater => rule!(deny_all);
                }
            })
            .globalize();

            (global, swappy_badge)
        }

        pub fn protected_method(&self) {
            Runtime::assert_access_rule(self.access_rule.clone())
        }
    }
}
