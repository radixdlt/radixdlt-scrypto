use crate::utils::*;
use scrypto::core::*;
use scrypto::resource::*;
use scrypto::types::*;
use scrypto::*;

blueprint! {
    struct ComponentTest {
        resource_def: ResourceDef,
        bucket: Vault,
        secret: String,
    }

    impl ComponentTest {
        pub fn create_component() -> Address {
            let resource_def = create_mutable("c1", Context::package_address());
            let bucket =  resource_def.mint(100);

            Self {
                resource_def,
                bucket: Vault::with_bucket(bucket),
                secret: "Secret".to_owned(),
            }.instantiate()
        }

        pub fn get_component_blueprint(address: Address) -> Blueprint {
            Component::from(address).blueprint()
        }

        pub fn get_component_state(&self) -> String {
            self.secret.clone()
        }

        pub fn put_component_state(&mut self)  {
            let bucket = self.resource_def.mint(100);

            // Receive bucket
            self.bucket.put(bucket);

            // Update state
            self.secret = "New secret".to_owned();
        }
    }
}
