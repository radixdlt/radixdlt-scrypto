use scrypto::prelude::*;

#[blueprint]
mod component_test {
    struct ComponentTest {
        test_vault: Vault,
        secret: String,
    }

    impl ComponentTest {
        fn create_test_token(amount: u32) -> Bucket {
            ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata(metadata! {
                    init {
                        "name" => "TestToken".to_owned(), locked;
                    }
                })
                .burn_roles(burn_roles! {
                    burner => rule!(allow_all);
                    burner_updater => rule!(deny_all);
                })
                .mint_initial_supply(amount)
                .into()
        }

        pub fn create_component() -> Global<ComponentTest> {
            Self {
                test_vault: Vault::with_bucket(Self::create_test_token(1000)),
                secret: "Secret".to_owned(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn get_component_state(&self) -> String {
            self.secret.clone()
        }

        pub fn put_component_state(&mut self) -> Bucket {
            // Take resource from vault
            let bucket = self.test_vault.take(1);

            // Update state
            self.secret = "New secret".to_owned();

            bucket
        }

        pub fn burn_bucket(&mut self, bucket: Bucket) {
            bucket.burn();
        }

        pub fn blueprint_name_function() -> String {
            Runtime::blueprint_name()
        }

        pub fn blueprint_name_method(&self) -> String {
            Runtime::blueprint_name()
        }
    }
}

#[derive(Debug, PartialEq, Eq, ScryptoSbor, NonFungibleData)]
pub struct ComponentTest2NonFungible {
    pub val: String,
}

#[blueprint]
mod component_test2 {

    use component_test3::ComponentTest3;

    struct ComponentTest2 {
        vault: NonFungibleVault,
    }

    impl ComponentTest2 {
        pub fn create_component() -> Global<ComponentTest2> {
            let resource_manager = ResourceBuilder::new_integer_non_fungible::<
                ComponentTest2NonFungible,
            >(OwnerRole::None)
            .mint_roles(mint_roles! {
                minter => rule!(allow_all);
                minter_updater => rule!(deny_all);
            })
            .burn_roles(burn_roles! {
                burner => rule!(allow_all);
                burner_updater => rule!(deny_all);
            })
            .create_with_no_initial_supply();

            let vault = resource_manager.create_empty_vault();

            Self { vault }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn generate_nft(&mut self) -> Bucket {
            let resource_manager = self.vault.resource_manager();

            let id = NonFungibleLocalId::integer(1);

            let bucket = resource_manager.mint_non_fungible(
                &id,
                ComponentTest2NonFungible {
                    val: String::from("FirstNFT"),
                },
            );
            self.vault.put(bucket);
            let bucket = self.vault.take_non_fungible(&id);
            bucket.into()
        }

        pub fn generate_nft_proof(&mut self) -> (Bucket, Proof) {
            let bucket = self.generate_nft();
            let proof = bucket.create_proof_of_all();
            (bucket, proof)
        }

        pub fn pass_vault_to_new_component(&mut self) -> Global<ComponentTest3> {
            let mut vault = Vault::with_bucket(self.generate_nft());
            let bucket = vault.take(dec!(1));
            let proof = bucket.create_proof_of_all();

            let return_value = ComponentTest3::create_component_with_vault_and_proof(vault, proof);

            bucket.burn();

            return_value
        }
    }
}

#[blueprint]
mod component_test3 {

    struct ComponentTest3 {
        vault: Vault,
    }

    impl ComponentTest3 {
        pub fn create_component(resource_id: ResourceAddress) -> Global<ComponentTest3> {
            Self {
                vault: Vault::new(resource_id),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn create_component_with_vault_and_proof(
            vault: Vault,
            proof: Proof,
        ) -> Global<ComponentTest3> {
            proof.check(vault.resource_address()).drop();
            Self { vault }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn check_proof(&self, proof: Proof) {
            proof.check(self.vault.resource_address()).drop();
        }

        pub fn check_proof_and_burn_bucket(&self, bucket: Bucket, proof: Proof) {
            proof.check(self.vault.resource_address()).drop();
            bucket.burn();
        }

        pub fn burn_bucket_and_check_proof(&self, bucket: Bucket, proof: Proof) {
            bucket.burn();
            proof.check(self.vault.resource_address()).drop();
        }
    }
}
