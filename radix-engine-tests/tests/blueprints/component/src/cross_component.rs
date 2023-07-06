use scrypto::prelude::*;

#[blueprint]
mod cross_component {
    enable_method_auth! {
        methods {
            put_auth => PUBLIC;
            cross_component_call => PUBLIC;
            get_component_state => restrict_to: [OWNER];
        }
    }

    struct CrossComponent {
        secret: String,
        auth_vault: Option<Vault>,
    }

    impl CrossComponent {
        pub fn create_component_with_auth(access_rule: AccessRule) -> Global<CrossComponent> {
            Self {
                secret: "Secret".to_owned(),
                auth_vault: None,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(access_rule))
            .globalize()
        }

        pub fn create_component() -> Global<CrossComponent> {
            Self::create_component_with_auth(rule!(allow_all))
        }

        pub fn put_auth(&mut self, mut auth_bucket: Vec<Bucket>) {
            self.auth_vault = Some(Vault::with_bucket(auth_bucket.remove(0)));
        }

        pub fn cross_component_call(&mut self, other_component: Global<CrossComponent>) -> String {
            match &mut self.auth_vault {
                Some(vault) => {
                    let auth_bucket = vault.take_all();
                    let value =
                        auth_bucket.authorize_with_all(|| other_component.get_component_state());
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
