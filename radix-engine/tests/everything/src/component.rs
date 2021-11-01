use scrypto::blueprint;
use scrypto::core::{Blueprint, Component, State};
use scrypto::resource::{Bucket, ResourceBuilder, Vault};
use scrypto::types::Address;

blueprint! {
    struct ComponentTest {
        test_vault: Vault,
        secret: String,
    }

    impl ComponentTest {
        pub fn create_component() -> Component {
            let bucket = ResourceBuilder::new()
                .metadata("name", "TestToken")
                .create_fixed(1000);

            Self {
                test_vault: Vault::with_bucket(bucket),
                secret: "Secret".to_owned(),
            }.instantiate()
        }

        pub fn get_component_blueprint(address: Address) -> Blueprint {
            Component::from(address).blueprint()
        }

        pub fn get_component_state(&self) -> String {
            self.secret.clone()
        }

        pub fn put_component_state(&mut self) -> Bucket  {
            // Take resource from vault
            let bucket = self.test_vault.take(1);

            // Update state
            self.secret = "New secret".to_owned();

            bucket
        }
    }
}
