use scrypto::prelude::*;

#[blueprint]
mod big_fi {
    use crate::swappy::Swappy;
    use crate::subservio::Subservio;
    use crate::subservio::SubservioFunctions;

    struct BigFi {
        child: Owned<Subservio>,
        swappy: Global<Swappy>,
        cerb_vault: Vault,
    }

    impl BigFi {
        pub fn create(cerb_resource: ResourceAddress, swappy: Global<Swappy>) -> (Global<BigFi>, NonFungibleBucket) {
            let big_fi_badge = ResourceBuilder::new_integer_non_fungible(OwnerRole::None)
                .mint_initial_supply(vec![(0u64.into(), ())]);

            let big_fi_resource = big_fi_badge.resource_address();

            let child = Blueprint::<Subservio>::create(cerb_resource);

            let cerb_vault = Vault::new(cerb_resource);

            let global = Self { child, swappy, cerb_vault }
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

        pub fn call_swappy(&self) {
            self.swappy.protected_method();
        }

        pub fn deposit_cerb(&mut self, cerbs: Bucket) {
            self.cerb_vault.put(cerbs);
        }

        pub fn deposit_cerb_into_subservio(&mut self, cerbs: Bucket) {
            self.child.deposit_cerb(cerbs);
        }
    }
}

#[blueprint]
mod subservio {
    struct Subservio {
        cerb_vault: Vault,
    }

    impl Subservio {
        pub fn create(cerb_resource: ResourceAddress) -> Owned<Subservio> {
            let cerb_vault = Vault::new(cerb_resource);
            Self { cerb_vault }.instantiate()
        }

        pub fn deposit_cerb(&mut self, cerbs: Bucket) {
            self.cerb_vault.put(cerbs);
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
