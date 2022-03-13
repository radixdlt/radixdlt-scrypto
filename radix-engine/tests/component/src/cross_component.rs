use scrypto::prelude::*;

blueprint! {
    struct CrossComponent {
        secret: String,
        auth_vault: Option<Vault>,
    }

    impl CrossComponent {
        pub fn create_component_with_auth(resource_def_id: ResourceDefId, non_fungible_id: NonFungibleId) -> ComponentId {
            let auth = NonFungibleAddress::new(resource_def_id, non_fungible_id);
            Self {
                secret: "Secret".to_owned(),
                auth_vault: None,
            }
            .instantiate_with_auth(HashMap::from([("get_component_state".to_string(), auth)]))
        }

        pub fn create_component() -> ComponentId {
            Self {
                secret: "Secret".to_owned(),
                auth_vault: None,
            }
            .instantiate()
        }

        pub fn cross_component_call(&self, component_id: ComponentId) -> String {
            let other_component = component!(component_id);
            other_component.call("get_component_state", vec![])
        }

        pub fn get_component_state(&self) -> String {
            self.secret.clone()
        }
    }
}
