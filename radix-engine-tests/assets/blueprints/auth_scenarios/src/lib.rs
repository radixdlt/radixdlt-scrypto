use scrypto::prelude::*;

#[blueprint]
mod big_fi {
    use crate::subservio::Subservio;
    use crate::subservio::SubservioFunctions;
    use crate::swappy::Swappy;
    use crate::swappy::SwappyFunctions;

    struct BigFi {
        child: Owned<Subservio>,
        swappy: Global<Swappy>,
        cerb: NonFungibleResourceManager,
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

            let child = Blueprint::<Subservio>::create(swappy, cerb_resource);

            let cerb_vault = Vault::new(cerb_resource.into());

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

        pub fn call_swappy_func(swappy: Global<Swappy>) {
            swappy.protected_method();
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
                .into()
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
            self.swappy
                .set_metadata_role("metadata_setter", AccessRule::AllowAll);
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

        pub fn call_swappy_function(&self) {
            Blueprint::<Swappy>::protected_function();
        }

        pub fn burn_bucket(&self, bucket: Bucket) {
            bucket.burn();
        }

        pub fn burn_vault(&mut self) {
            self.cerb_vault.burn(1);
        }

        pub fn assert_in_subservio(&mut self, swappy_badge: Bucket) -> Bucket {
            let bucket = self.cerb_vault.take(1);
            let swappy_badge = bucket.authorize_with_all(|| self.child.assert_local(swappy_badge));
            self.cerb_vault.put(bucket);
            swappy_badge
        }

        pub fn call_swappy_in_subservio(&mut self, swappy_badge: Bucket) -> Bucket {
            let bucket = self.cerb_vault.take(1);
            let swappy_badge = bucket.authorize_with_all(|| self.child.call_swappy(swappy_badge));
            self.cerb_vault.put(bucket);
            swappy_badge
        }

        pub fn pass_proof(&mut self, proof: Proof) {
            self.child.receive_proof(proof);
        }

        pub fn create_and_pass_proof(&mut self) {
            let proof = self
                .cerb_vault
                .as_non_fungible()
                .create_proof_of_non_fungibles(&indexset!(NonFungibleLocalId::integer(1)));
            self.child.receive_and_pass_proof(proof.into());
        }
    }
}

#[blueprint]
mod subservio {
    use crate::swappy::Swappy;

    struct Subservio {
        swappy: Global<Swappy>,
        cerb_vault: Vault,
    }

    impl Subservio {
        pub fn create(swappy: Global<Swappy>, cerb_resource: ResourceAddress) -> Owned<Subservio> {
            let cerb_vault = Vault::new(cerb_resource.into());

            Self { swappy, cerb_vault }.instantiate()
        }

        pub fn deposit_cerb(&mut self, cerbs: Bucket) {
            self.cerb_vault.put(cerbs);
        }

        pub fn assert_local(&self, swappy_badge: Bucket) -> Bucket {
            let proof = swappy_badge.create_proof_of_all();
            let cerb = self.cerb_vault.resource_address();
            let swappy = swappy_badge.resource_address();
            LocalAuthZone::push(proof);

            Runtime::assert_access_rule(rule!(require(cerb) && require(swappy)));

            swappy_badge
        }

        pub fn call_swappy(&self, swappy_badge: Bucket) -> Bucket {
            let proof = swappy_badge.create_proof_of_all();
            LocalAuthZone::push(proof);

            self.swappy.method_protected_by_cerb_and_swappy();

            swappy_badge
        }

        pub fn receive_proof(&self, _proof: Proof) {}

        pub fn receive_and_pass_proof(&self, proof: Proof) {
            self.swappy.receive_proof(proof);
        }
    }
}

#[blueprint]
mod swappy {
    enable_function_auth! {
        create => rule!(allow_all);
        protected_function => rule!(require(XRD));
    }

    enable_method_auth! {
        roles {
            some_role => updatable_by: [];
            other_role => updatable_by: [];
        },
        methods {
            public_method => PUBLIC;
            put_proof_in_auth_zone => PUBLIC;
            set_metadata => PUBLIC;
            update_metadata_rule => PUBLIC;
            receive_proof => PUBLIC;
            protected_method => restrict_to: [some_role];
            another_protected_method => restrict_to: [some_role];
            another_protected_method2 => restrict_to: [some_role];
            method_protected_by_cerb_and_swappy => restrict_to: [other_role];
        }
    }

    struct Swappy {
        access_rule: AccessRule,
    }

    impl Swappy {
        pub fn create(cerb: ResourceAddress) -> (Global<Swappy>, NonFungibleBucket) {
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
                other_role => rule!(require(cerb) && require(swappy_resource));
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

        pub fn protected_function() {}

        pub fn method_protected_by_cerb_and_swappy(&self) {}

        pub fn receive_proof(&self, _proof: Proof) {}
    }
}

#[blueprint]
mod count_of_zero {
    enable_function_auth! {
        hi => AccessRule::Protected(CompositeRequirement::BasicRequirement(BasicRequirement::CountOf(0, vec![ResourceOrNonFungible::Resource(XRD)])));
    }

    struct CountOfZero {}

    impl CountOfZero {
        pub fn hi() {}
    }
}
