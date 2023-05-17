use scrypto::prelude::*;

#[blueprint]
mod factory {
    struct Factory {
        my_address: Option<ComponentAddress>,
    }

    impl Factory {
        pub fn create_raw() -> ComponentAddress {
            let component = Self {
                my_address: Option::None,
            }
            .instantiate();

            let mut method_authorities = MethodAuthorities::new();
            method_authorities.set_main_method_authority("set_address", "set_address");

            let mut authority_rules = AuthorityRules::new();
            authority_rules.set_main_rule(
                "set_address",
                rule!(require(Runtime::package_token())),
                AccessRule::DenyAll,
            );

            component.globalize_with_access_rules(method_authorities, authority_rules)
        }

        pub fn create() -> ComponentAddress {
            let component = Self {
                my_address: Option::None,
            }
            .instantiate();

            let mut method_authorities = MethodAuthorities::new();
            method_authorities.set_main_method_authority("set_address", "set_address");

            let mut authority_rules = AuthorityRules::new();
            authority_rules.set_main_rule(
                "set_address",
                rule!(require(Runtime::package_token())),
                AccessRule::DenyAll,
            );

            let component_address =
                component.globalize_with_access_rules(method_authorities, authority_rules);
            let component_ref: FactoryGlobalComponentRef = component_address.into();
            component_ref.set_address(component_address);

            component_address
        }

        pub fn set_address(&mut self, my_address: ComponentAddress) {
            self.my_address = Option::Some(my_address);
        }
    }
}
