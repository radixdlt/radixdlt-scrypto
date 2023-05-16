use scrypto::prelude::*;

#[blueprint]
mod cross_component {
    struct CrossComponent {
        secret: String,
        auth_vault: Option<Vault>,
    }

    impl CrossComponent {
        pub fn create_component_with_auth(
            authority_rules: AuthorityRules,
        ) -> Global<CrossComponent> {
            Self {
                secret: "Secret".to_owned(),
                auth_vault: None,
            }
            .instantiate()
            .method_authority("get_component_state", "auth")
            .authority_rules(authority_rules)
            .globalize()
        }

        pub fn create_component() -> Global<CrossComponent> {
            let component = Self {
                secret: "Secret".to_owned(),
                auth_vault: None,
            }
            .instantiate();
            component.globalize()
        }

        pub fn put_auth(&mut self, mut auth_bucket: Vec<Bucket>) {
            self.auth_vault = Some(Vault::with_bucket(auth_bucket.remove(0)));
        }

        pub fn cross_component_call(&mut self, other_component: Global<CrossComponent>) -> String {
            match &mut self.auth_vault {
                Some(vault) => {
                    let auth_bucket = vault.take_all();
                    let value = auth_bucket.authorize(|| other_component.get_component_state());
                    vault.put(auth_bucket);
                    value
                }
                None => other_component.get_component_state(),
            }
        }

        pub fn get_component_state(&self) -> String {
            self.secret.clone()
        }
    }
}
