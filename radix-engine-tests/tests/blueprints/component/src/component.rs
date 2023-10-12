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

        pub fn take_resource_amount_of_bucket(&mut self, bucket: Bucket) -> (Bucket, Bucket) {
            let bucket_ret = self.test_vault.take(1 + bucket.amount());
            (bucket_ret, bucket)
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

    struct ComponentTest2 {
        vault: Vault,
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
            let bucket = self.vault.as_non_fungible().take_non_fungible(&id);
            bucket.into()
        }
    }
}

#[blueprint]
mod component_test3 {

    struct ComponentTest3 {
        resource_id: ResourceAddress,
    }

    impl ComponentTest3 {
        pub fn create_component(resource_id: ResourceAddress) -> Global<ComponentTest3> {
            Self { resource_id }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn check(&self, proof: Proof) {
            proof.check(self.resource_id).drop();
        }
    }
}
