use scrypto::prelude::*;

#[blueprint]
mod cross_component {
    struct CrossComponent {
        secret: String,
        auth_vault: Option<Vault>,
    }

    impl CrossComponent {
        pub fn create_component_with_auth(access_rules: AccessRulesConfig) -> ComponentAddress {
            let component = Self {
                secret: "Secret".to_owned(),
                auth_vault: None,
            }
            .instantiate();
            component.globalize_with_access_rules(access_rules)
        }

        pub fn create_component() -> ComponentAddress {
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

        pub fn cross_component_call(&mut self, component_address: ComponentAddress) -> String {
            let other_component_ref: CrossComponentGlobalComponentRef = component_address.into();
            match &mut self.auth_vault {
                Some(vault) => {
                    let auth_bucket = vault.take_all();
                    let value = auth_bucket.authorize(|| other_component_ref.get_component_state());
                    vault.put(auth_bucket);
                    value
                }
                None => other_component_ref.get_component_state(),
            }
        }

        pub fn get_component_state(&self) -> String {
            self.secret.clone()
        }
    }
}
