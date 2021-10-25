use scrypto::blueprint;
use scrypto::core::{Blueprint, Component, State};
use scrypto::resource::{ResourceDef, Vault};
use scrypto::types::Address;

use crate::utils::*;

blueprint! {
    struct ComponentTest {
        resource_def: ResourceDef,
        mint_auth: Vault,
        bucket: Vault,
        secret: String,
    }

    impl ComponentTest {
        pub fn create_component() -> Component {
            let (resource_def, auth) = create_mutable("c1");
            let bucket =  resource_def.mint(100, auth.borrow());

            Self {
                resource_def,
                mint_auth: Vault::with_bucket(auth),
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
            let auth = self.mint_auth.take(1);
            let bucket = self.resource_def.mint(100,auth.borrow());
            self.mint_auth.put(auth);

            // Receive bucket
            self.bucket.put(bucket);

            // Update state
            self.secret = "New secret".to_owned();
        }
    }
}
