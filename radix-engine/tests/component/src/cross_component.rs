use scrypto::prelude::*;

blueprint! {
    struct CrossComponent {
        secret: String,
        auth_vault: Option<Vault>,
    }

    impl CrossComponent {
        pub fn create_component_with_auth(
            auth: Authorization,
        ) -> ComponentAddress {
            Self {
                secret: "Secret".to_owned(),
                auth_vault: None,
            }
            .instantiate()
            .auth(auth)
            .globalize()
        }

        pub fn create_component() -> ComponentAddress {
            Self {
                secret: "Secret".to_owned(),
                auth_vault: None,
            }
            .instantiate()
            .globalize()
        }

        pub fn put_auth(&mut self, mut auth_bucket: Vec<Bucket>) {
            self.auth_vault = Some(Vault::with_bucket(auth_bucket.remove(0)));
        }

        pub fn cross_component_call(&mut self, component_address: ComponentAddress) -> String {
            let other_component = component!(component_address);
            match &mut self.auth_vault {
                Some(vault) => {
                    let auth_bucket = vault.take_all();
                    let value = auth_bucket
                        .authorize(|| other_component.call("get_component_state", vec![]));
                    vault.put(auth_bucket);
                    value
                }
                None => other_component.call("get_component_state", vec![]),
            }
        }

        pub fn get_component_state(&self) -> String {
            self.secret.clone()
        }
    }
}
