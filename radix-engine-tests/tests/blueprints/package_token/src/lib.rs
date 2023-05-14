use scrypto::prelude::*;

#[blueprint]
mod factory {
    struct Factory {
        my_component: Option<Global<FactoryComponent>>,
    }

    impl Factory {
        pub fn create_raw() -> Global<FactoryComponent> {
            let component = Self {
                my_component: Option::None,
            }
            .instantiate();

            let mut method_authorities = MethodAuthorities::new();
            method_authorities.set_main_method_authority("set_address", "set_address");

            let mut authority_rules = AuthorityRules::new();
            authority_rules.set_rule(
                "set_address",
                rule!(require(Runtime::package_token())),
                AccessRule::DenyAll,
            );

            component.globalize_with_access_rules(method_authorities, authority_rules)
        }

        pub fn create() -> Global<FactoryComponent> {
            let component = Self {
                my_component: Option::None,
            }
            .instantiate();

            let mut method_authorities = MethodAuthorities::new();
            method_authorities.set_main_method_authority("set_address", "set_address");

            let mut authority_rules = AuthorityRules::new();
            authority_rules.set_rule(
                "set_address",
                rule!(require(Runtime::package_token())),
                AccessRule::DenyAll,
            );

            let component =
                component.globalize_with_access_rules(method_authorities, authority_rules);
            component.set_address(component.clone());

            component
        }

        pub fn set_address(&mut self, my_component: Global<FactoryComponent>) {
            self.my_component = Option::Some(my_component);
        }
    }
}
