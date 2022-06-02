use scrypto::prelude::*;

#[derive(NonFungibleData)]
pub struct User {
    pub name: String,
}

blueprint! {
    struct Hello {
        // Define what resources and data will be managed by Hello components
        sample_vault: Vault,
        nf_badge_vault: Vault,
        access_rules: AccessRules,
    }

    impl Hello {
        pub fn badge_for_both(&self) -> Proof {
            let p = self.nf_badge_vault.create_proof(); // by returning it someone pushes it, not sure who
            info!(
                "nf_badge_for returning: {} {:?}",
                p.resource_address(),
                p.non_fungible_ids()
            );
            p
        }
        pub fn badge_for_alice(&self) -> Proof {
            let mut ids = BTreeSet::new();
            ids.insert(NonFungibleId::from_u64(1));
            let p = self.nf_badge_vault.create_proof_by_ids(&ids); // by returning it someone pushes it, not sure who
            info!(
                "badge_for_alice returning: {} {:?}",
                p.resource_address(),
                p.non_fungible_ids()
            );
            p
        }
        pub fn badge_for_bob(&self) -> Proof {
            let mut ids = BTreeSet::new();
            ids.insert(NonFungibleId::from_u64(2));
            let p = self.nf_badge_vault.create_proof_by_ids(&ids); // by returning it someone pushes it, not sure who
            info!(
                "badge_for_bob returning: {} {:?}",
                p.resource_address(),
                p.non_fungible_ids()
            );
            p
        }
        pub fn instantiate_hello() -> ComponentAddress {
            // Create a new token called "HelloToken," with a fixed supply of 1000, and put that supply into a bucket
            let my_bucket: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "HelloToken")
                .metadata("symbol", "HT")
                .initial_supply(dec!(1000u32));

            let nf_admin_badge: Bucket = ResourceBuilder::new_non_fungible().initial_supply([
                (
                    NonFungibleId::from_u64(1),
                    User {
                        name: "Alice".to_owned(),
                    },
                ),
                (
                    NonFungibleId::from_u64(2),
                    User {
                        name: "Bob".to_owned(),
                    },
                ),
            ]);

            let access_rules = AccessRules::new()
                .method(
                    "free_token",
                    rule!(require(nf_admin_badge.resource_address())),
                ) // limit to only those with badge (and record them in CallerAuthZone)
                .default(rule!(allow_all));

            // Instantiate a Hello component, populating its vault with our supply of 1000 HelloToken
            let my_component = Self {
                sample_vault: Vault::with_bucket(my_bucket),
                nf_badge_vault: Vault::with_bucket(nf_admin_badge),
                access_rules: access_rules.clone(), // keeping a local copy for internal checks
            }
            .instantiate();
            // Apply my rules, and add my component to global address space so it can be called by others
            let component_address = my_component.add_access_check(access_rules).globalize();

            component_address
        }

        pub fn free_token(&mut self) -> Bucket {
            // get the NF proof that is required even though it was passed via AuthZone
            info!("free_token called");
            let ra = self.nf_badge_vault.resource_address();
            let proof = CallerAuthZone::create_proof(ra);
            let count = proof.non_fungible_ids().len();
            info!(
                "nf_free_token got some nf_badge_vault proof with count: {}",
                count
            );
            assert_eq!(count, 1, "CallerAuthZone had too many NFTs in proof");
            let user: User = proof.non_fungibles()[0].data(); // only care about the first one (this could cause problems if the create_proof order is not consistent and deterministic)
            let take_amount = if user.name == "Alice" {
                dec!(10)
            } else {
                dec!(1)
            };
            info!(
                "My balance is: {} HelloToken. Now giving away {} tokens to {}",
                self.sample_vault.amount(),
                take_amount,
                user.name
            );
            self.sample_vault.take(take_amount)
        }
    }
}

mod caller {
    use scrypto::prelude::*;

    blueprint! {
        struct Caller {
            tokens: Option<Vault>,
        }
        impl Caller {
            pub fn instantiate_caller() -> ComponentAddress {
                // Create a new token called "HelloToken," with a fixed supply of 1000, and put that supply into a bucket
                Self { tokens: None }.instantiate().globalize()
            }

            pub fn run_as_alice(&mut self, callee: ComponentAddress) {
                info!("running");
                ComponentAuthZone::start();
                let hello: super::super::Hello = callee.into();
                let alice_badge: Proof = hello.badge_for_alice();
                ComponentAuthZone::push(alice_badge);
                let free_tokens = hello.free_token();
                ComponentAuthZone::end();
                if let Some(vault) = &mut self.tokens {
                    vault.put(free_tokens);
                } else {
                    self.tokens = Some(Vault::with_bucket(free_tokens));
                }
            }

            pub fn xfail_run_as_alice(&mut self, callee: ComponentAddress) {
                info!("running");
                // fails without non-default auth zone
                // ComponentAuthZone::start();
                let hello: super::super::Hello = callee.into();
                let alice_badge: Proof = hello.badge_for_alice();
                ComponentAuthZone::push(alice_badge);
                let free_tokens = hello.free_token();
                // ComponentAuthZone::end();
                if let Some(vault) = &mut self.tokens {
                    vault.put(free_tokens);
                } else {
                    self.tokens = Some(Vault::with_bucket(free_tokens));
                }
            }

            pub fn run_as_both(&mut self, callee: ComponentAddress) {
                info!("running");
                ComponentAuthZone::start(); // use non-default

                let hello: super::super::Hello = callee.into();
                let alice_badge: Proof = hello.badge_for_alice();
                let bob_badge: Proof = hello.badge_for_bob();

                ComponentAuthZone::push(alice_badge);
                let alice_tokens = hello.free_token();

                ComponentAuthZone::start(); // fresh auth zone
                ComponentAuthZone::push(bob_badge);
                let bob_tokens = hello.free_token();
                ComponentAuthZone::end(); // end auth zone

                // more for alice
                let more_alice_tokens = hello.free_token();

                ComponentAuthZone::end(); // end non-default

                assert_eq!(alice_tokens.amount(), dec!(10));
                assert_eq!(bob_tokens.amount(), dec!(1));
                assert_eq!(more_alice_tokens.amount(), dec!(10));

                if let Some(vault) = &mut self.tokens {
                    vault.put(alice_tokens);
                    vault.put(bob_tokens);
                    vault.put(more_alice_tokens);
                } else {
                    let mut vault = Vault::new(alice_tokens.resource_address());
                    vault.put(alice_tokens);
                    vault.put(bob_tokens);
                    vault.put(more_alice_tokens);
                    self.tokens = Some(vault);
                }
            }
        }
    }
}
