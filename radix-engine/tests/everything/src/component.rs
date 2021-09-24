use crate::utils::*;
use scrypto::constructs::*;
use scrypto::resource::*;
use scrypto::types::*;
use scrypto::*;

blueprint! {
    struct ComponentTest {
        resource: Address,
        bucket: Vault,
        secret: String,
    }

    impl ComponentTest {
        pub fn create_component() -> Address {
            let resource = create_mutable("c1", Context::package_address());
            let bucket =  mint_resource(resource, 100);

            Self {
                resource,
                bucket: Vault::wrap(bucket),
                secret: "Secret".to_owned(),
            }.instantiate()
        }

        pub fn get_component_info(address: Address) -> Blueprint {
            Component::from(address).blueprint()
        }

        pub fn get_component_state(&self) -> String {
            self.secret.clone()
        }

        pub fn put_component_state(&mut self)  {
            let bucket = mint_resource(self.resource, 100);

            // Receive resource
            self.bucket.put(bucket);

            // Update state
            self.secret = "New secret".to_owned();
        }
    }
}
