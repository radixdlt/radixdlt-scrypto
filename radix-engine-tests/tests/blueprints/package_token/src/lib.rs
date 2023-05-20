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
                   roles.define_role("set_address", rule!(require(Runtime::package_token())), rule!(deny_all));
                   roles
                })
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
                roles.define_role("set_address", rule!(require(Runtime::package_token())), rule!(deny_all));
                roles
            })
            .globalize();

            component.set_address(component.clone());

            component
        }

        #[restrict_to("set_address")]
        pub fn set_address(&mut self, my_component: Global<Factory>) {
            self.my_component = Option::Some(my_component);
        }
    }
}
