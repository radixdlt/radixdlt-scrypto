use scrypto::prelude::*;

#[blueprint]
mod big_fi {
    use crate::subservio::Subservio;
    use crate::subservio::SubservioFunctions;
    use crate::swappy::Swappy;

    struct BigFi {
        child: Owned<Subservio>,
        swappy: Global<Swappy>,
        cerb: ResourceManager,
        cerb_vault: Vault,
    }

    impl BigFi {
        pub fn create(
            cerb_resource: ResourceAddress,
            swappy: Global<Swappy>,
        ) -> (Global<BigFi>, NonFungibleBucket) {
            let big_fi_badge = ResourceBuilder::new_integer_non_fungible(OwnerRole::None)
                .mint_initial_supply(vec![(0u64.into(), ())]);

            let big_fi_resource = big_fi_badge.resource_address();

            let child = Blueprint::<Subservio>::create(cerb_resource);

            let cerb_vault = Vault::new(cerb_resource);

            let global = Self {
                child,
                swappy,
                cerb: cerb_resource.into(),
                cerb_vault,
            }
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

        pub fn mint_cerb(&self) -> Bucket {
            self.cerb
                .mint_non_fungible(&NonFungibleLocalId::Integer(64u64.into()), ())
        }

        pub fn recall_cerb(&self, vault_id: InternalAddress) -> Bucket {
            let bucket: Bucket = scrypto_decode(&ScryptoVmV1Api::object_call_direct(
                vault_id.as_node_id(),
                VAULT_RECALL_IDENT,
                scrypto_args!(Decimal::ONE),
            ))
            .unwrap();

            bucket
        }

        pub fn set_swappy_metadata(&self) {
            self.swappy.set_metadata("key", "value".to_string());
        }

        pub fn update_swappy_metadata_rule(&self) {
            self.swappy.set_metadata_role("metadata_setter", AccessRule::AllowAll);
        }

        pub fn update_cerb_rule(&self) {
            self.cerb.set_role("withdrawer", AccessRule::AllowAll);
        }

        pub fn some_method(&self) {}

        pub fn some_function() {}

        pub fn call_swappy_with_badge(&self, bucket: Bucket) -> Bucket {
            bucket.authorize_with_all(|| {
                let bigfi_self: Global<BigFi> = Runtime::global_address().into();
                bigfi_self.some_method();
                Blueprint::<BigFi>::some_function();
                self.call_swappy();
            });

            bucket
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
            public_method => PUBLIC;
            put_proof_in_auth_zone => PUBLIC;
            set_metadata => PUBLIC;
            update_metadata_rule => PUBLIC;
            protected_method => restrict_to: [some_role];
            another_protected_method => restrict_to: [some_role];
            another_protected_method2 => restrict_to: [some_role];
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
                    metadata_setter_updater => rule!(require(swappy_resource));
                }
            })
            .globalize();

            (global, swappy_badge)
        }

        pub fn protected_method(&self) {
            Runtime::assert_access_rule(self.access_rule.clone());
        }

        pub fn another_protected_method(&self) {
            self.protected_method();
        }

        pub fn another_protected_method2(&self) {
            let me: Global<Swappy> = Runtime::global_address().into();
            me.protected_method();
        }

        pub fn public_method(&self, _proof: Proof) {
            Runtime::assert_access_rule(self.access_rule.clone());
        }

        pub fn put_proof_in_auth_zone(&self, proof: Proof) {
            LocalAuthZone::push(proof);
        }

        pub fn set_metadata(&self) {
            Runtime::global_component().set_metadata("key", "value".to_string());
        }

        pub fn update_metadata_rule(&self) {
            Runtime::global_component().set_metadata_role("metadata_setter", AccessRule::AllowAll);
        }
    }
}
