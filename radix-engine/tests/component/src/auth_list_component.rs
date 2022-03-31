use scrypto::prelude::*;

blueprint! {
    struct AuthListComponent {
        auth: Vec<NonFungibleAddress>,
    }

    impl AuthListComponent {
        pub fn create_component(
            auth: Vec<NonFungibleAddress>,
            authorization: ComponentAuthorization,
        ) -> ComponentId {
            Self { auth }.globalize_auth(authorization)
        }

        pub fn update_auth(&mut self, auth: Vec<NonFungibleAddress>) {
            self.auth = auth;
        }

        pub fn get_secret(&self) -> String {
            "Secret".to_owned()
        }
    }
}
