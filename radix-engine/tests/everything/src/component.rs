use crate::utils::*;
use scrypto::constructs::*;
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

            Self {
                resource: resource,
                tokens: tokens,
                secret: "Secret".to_owned(),
            }.instantiate()
        }

        pub fn get_component_info(address: Address) -> ComponentInfo {
            Component::from(address).get_info()
        }

        pub fn get_component_state(&self) -> String {
            self.secret.clone()
        }

        pub fn put_component_state(&mut self)  {
            let tokens = mint_tokens(self.resource, 100);

            // Receive resource
            self.tokens.put(tokens);

            // Update state
            self.secret = "New secret".to_owned();
        }

        pub fn test_component_map(&self) -> Option<String> {
            let address = Self {
                resource: Address::RadixToken,
                tokens: Tokens::new_empty(Address::RadixToken),
                secret: "Secret".to_owned(),
            }.instantiate();
            let component = Component::from(address);

            component.put_map_entry("hello", "world");
            component.get_map_entry("hello")
        }
    }
}
