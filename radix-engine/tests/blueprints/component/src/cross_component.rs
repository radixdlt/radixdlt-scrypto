use scrypto::prelude::*;

blueprint! {
    struct CrossComponent {
        secret: String,
        auth_vault: Option<Vault>,
    }

    impl CrossComponent {
        pub fn create_component_with_auth(access_rules: AccessRules) -> ComponentAddress {
            let mut component = Self {
                secret: "Secret".to_owned(),
                auth_vault: None,
            }
            .instantiate();
            component.add_access_check(access_rules);
            component.globalize()
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
            let other_component = borrow_component!(component_address);
            match &mut self.auth_vault {
                Some(vault) => {
                    let auth_bucket = vault.take_all();
                    let value = auth_bucket
                        .authorize(|| other_component.call("get_component_state", args![]));
                    vault.put(auth_bucket);
                    value
                }
                None => other_component.call("get_component_state", args![]),
            }
        }

        pub fn get_component_state(&self) -> String {
            self.secret.clone()
        }
    }
}
