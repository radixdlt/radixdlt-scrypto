use crate::utils::*;
use scrypto::constructs::*;
use scrypto::kernel::*;
use scrypto::resource::*;
use scrypto::types::*;
use scrypto::*;

blueprint! {
    struct ComponentTest {
        resource: Address,
        tokens: Tokens,
        secret: String,
    }

    impl ComponentTest {
        pub fn create_component() -> Address {
            let resource = create_mutable_tokens("c1", Context::package_address());
            let tokens  =  mint_tokens(resource, 100);

           Component::new("ComponentTest", Self {
               resource: resource,
               tokens: tokens,
               secret: "abc".to_owned(),
           }).into()
        }

        pub fn get_component_info(address: Address) -> ComponentInfo {
            Component::from(address).get_info()
        }

        pub fn get_component_state(&self) -> String {
            self.secret.clone()
        }

        pub fn put_component_state(&mut self)  {
            let resource: Resource = self.resource.clone().into();
            let tokens = resource.mint_tokens(U256::from(100));

            // Add some tokens
            self.tokens.put(tokens);

            // Update data
            self.secret = "New secret".to_owned();
        }
    }
}
