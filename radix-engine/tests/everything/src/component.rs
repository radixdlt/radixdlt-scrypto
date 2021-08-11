use scrypto::constructs::*;
use scrypto::kernel::*;
use scrypto::resource::*;
use scrypto::types::*;
use scrypto::*;

component! {
    struct ComponentTest {
        resource: Address,
        tokens: Tokens,
        secret: String,
    }

    impl ComponentTest {
        pub fn create_component() -> Address {
            let (resource, tokens) = Self::mint_tokens();

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

        pub fn mint_tokens() -> (Address, Tokens) {
            let owner = Context::address();
            info!("Owner address: {:?}", owner);

            let resource = Resource::new(
                "symbol",
                "name",
                "description",
                "url",
                "icon_url",
                Some(owner),
                Some(U256::from(1000))
            );
            let tokens = resource.mint_tokens(U256::from(500));

            (resource.into(), tokens)
        }
    }
}
