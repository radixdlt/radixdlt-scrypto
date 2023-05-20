use scrypto::prelude::*;

#[blueprint]
mod factory {
    struct Factory {
        my_component: Option<Global<Factory>>,
    }

    impl Factory {
        pub fn create_raw() -> Global<Factory> {
            Self { my_component: None }
                .instantiate()
                .prepare_to_globalize()
                .define_roles({
                   let mut roles = AuthorityRules::new();
                   roles.define_role("auth", rule!(require(Runtime::package_token())), rule!(deny_all));
                   roles
                })
                .protect_methods(btreemap!(
                    "set_address" => vec!["auth".to_string()],
                ))
                .globalize()
        }

        pub fn create() -> Global<Factory> {
            let component = Self {
                my_component: Option::None,
            }
            .instantiate()
            .prepare_to_globalize()
            .define_roles({
                let mut roles = AuthorityRules::new();
                roles.define_role("auth", rule!(require(Runtime::package_token())), rule!(deny_all));
                roles
            })
            .protect_methods(btreemap!(
                "set_address" => vec!["auth".to_string()],
            ))
            .globalize();

            component.set_address(component.clone());

            component
        }

        pub fn set_address(&mut self, my_component: Global<Factory>) {
            self.my_component = Option::Some(my_component);
        }
    }
}
