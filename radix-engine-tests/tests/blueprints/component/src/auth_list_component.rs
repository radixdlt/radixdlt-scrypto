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
        ) -> Global<AuthListComponentComponent> {
            let method_authorities = MethodAuthorities::new();
            let component = Self { count, auth }.instantiate();
            component.globalize_with_access_rules(method_authorities, authority_rules)
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
