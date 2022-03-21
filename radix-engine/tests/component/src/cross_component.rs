use scrypto::prelude::*;

blueprint! {
    struct CrossComponent {
        secret: String,
        auth_vault: Option<Vault>,
    }

    impl CrossComponent {
        pub fn create_component_with_auth(
            proof_rule: ProofRule,
        ) -> ComponentId {
            Self {
                secret: "Secret".to_owned(),
                auth_vault: None,
            }
            .instantiate_with_auth(HashMap::from([(
                "get_component_state".to_string(),
                proof_rule,
            )]))
        }

        pub fn create_component() -> ComponentId {
            Self {
                secret: "Secret".to_owned(),
                auth_vault: None,
            }
            .instantiate()
        }

        pub fn put_auth(&mut self, mut auth_bucket: Vec<Bucket>) {
            self.auth_vault = Some(Vault::with_bucket(auth_bucket.remove(0)));
        }

        pub fn cross_component_call(&mut self, component_id: ComponentId) -> String {
            let other_component = component!(component_id);
            match &mut self.auth_vault {
                Some(vault) => {
                    let auth_bucket = vault.take_all();
                    let value = authorize(&auth_bucket, || {
                        other_component.call("get_component_state", vec![])
                    });
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
