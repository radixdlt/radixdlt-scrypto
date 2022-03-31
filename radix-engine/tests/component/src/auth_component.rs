use scrypto::prelude::*;

blueprint! {
    struct AuthComponent {
        some_non_fungible: NonFungibleAddress,
    }

    impl AuthComponent {
        pub fn create_component(some_non_fungible: NonFungibleAddress) -> ComponentId {
            let mut component: LocalComponent = Self { some_non_fungible }.into();
            component.auth("get_secret", this!(SchemaPath::new().field("some_non_fungible")));
            component.globalize()
        }

        pub fn get_secret(&self) -> String {
            "Secret".to_owned()
        }

        pub fn update_auth(&mut self, some_non_fungible: NonFungibleAddress) {
            self.some_non_fungible = some_non_fungible;
        }
    }
}
