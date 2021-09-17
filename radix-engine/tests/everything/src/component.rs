use crate::utils::*;
use scrypto::constructs::*;
use scrypto::resource::*;
use scrypto::types::*;
use scrypto::*;

blueprint! {
    struct ComponentTest {
        resource: Address,
        bucket: Bucket,
        secret: String,
    }

    impl ComponentTest {
        pub fn create_component() -> Address {
            let resource = create_mutable("c1", Context::package_address());
            let bucket =  mint_bucket(resource, 100);

            Self {
                resource: resource,
                bucket: bucket,
                secret: "Secret".to_owned(),
            }.instantiate()
        }

        pub fn get_component_info(address: Address) -> ComponentInfo {
            Component::from(address).info()
        }

        pub fn get_component_state(&self) -> String {
            self.secret.clone()
        }

        pub fn put_component_state(&mut self)  {
            let bucket = mint_bucket(self.resource, 100);

            // Receive resource
            self.bucket.put(bucket);

            // Update state
            self.secret = "New secret".to_owned();
        }
    }
}
