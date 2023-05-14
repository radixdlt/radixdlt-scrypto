use scrypto::prelude::*;

#[blueprint]
mod factory {
    struct Factory {
        my_component: Option<Global<FactoryComponent>>,
    }

    impl Factory {
        pub fn create_raw() -> Global<FactoryComponent> {
            let component = Self { my_component: None }.instantiate();

            let access_rules = {
                let mut method_authorities = MethodAuthorities::new();
                method_authorities.set_main_method_authority("set_address", "set_address");

                let mut authority_rules = AuthorityRules::new();
                authority_rules.set_rule(
                    "set_address",
                    rule!(require(Runtime::package_token())),
                    AccessRule::DenyAll,
                );
                AccessRules::new(method_authorities, authority_rules)
            };

            component.attach_access_rules(access_rules).globalize()
        }

        pub fn create() -> Global<FactoryComponent> {
            let component = Self {
                my_component: Option::None,
            }
            .instantiate();

            let access_rules = {
                let mut method_authorities = MethodAuthorities::new();
                method_authorities.set_main_method_authority("set_address", "set_address");

                let mut authority_rules = AuthorityRules::new();
                authority_rules.set_rule(
                    "set_address",
                    rule!(require(Runtime::package_token())),
                    AccessRule::DenyAll,
                );
                AccessRules::new(method_authorities, authority_rules)
            };

            let component = component.attach_access_rules(access_rules).globalize();
            component.set_address(component.clone());

            component
        }

        pub fn set_address(&mut self, my_component: Global<FactoryComponent>) {
            self.my_component = Option::Some(my_component);
        }
    }
}
