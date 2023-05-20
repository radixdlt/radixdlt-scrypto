use scrypto::prelude::*;

#[blueprint]
mod auth_list_component {
    struct AuthListComponent {
        count: u8,
        auth: Vec<NonFungibleGlobalId>,
    }

    impl AuthListComponent {
        pub fn create_component(
            count: u8,
            auth: Vec<NonFungibleGlobalId>,
            authority_rules: AuthorityRules,
        ) -> Global<AuthListComponent> {
            Self { count, auth }
                .instantiate()
                .prepare_to_globalize()
                .define_roles(authority_rules)
                .globalize()
        }

        pub fn update_count(&mut self, count: u8) {
            self.count = count;
        }

        pub fn update_auth(&mut self, auth: Vec<NonFungibleGlobalId>) {
            self.auth = auth;
        }

        pub fn get_secret(&self) -> String {
            "Secret".to_owned()
        }
    }
}
