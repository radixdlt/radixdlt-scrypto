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

            let mut access_rules = AccessRulesConfig::new();
            access_rules.set_group_access_rule_and_mutability(
                "set_address",
                rule!(require(Runtime::package_token())),
                AccessRule::DenyAll,
            );
            access_rules.set_main_method_group("set_address", "set_address");

            component.globalize_with_access_rules(access_rules)
        }

        pub fn create() -> ComponentAddress {
            let component = Self {
                my_address: Option::None,
            }
            .instantiate();

            let mut access_rules = AccessRulesConfig::new();
            access_rules.set_group_access_rule_and_mutability(
                "set_address",
                rule!(require(Runtime::package_token())),
                AccessRule::DenyAll,
            );

            access_rules.set_main_method_group("set_address", "set_address");

            let component_address = component.globalize_with_access_rules(access_rules);
            let component_ref: FactoryGlobalComponentRef = component_address.into();
            component_ref.set_address(component_address);

            component_address
        }

        pub fn set_address(&mut self, my_address: ComponentAddress) {
            self.my_address = Option::Some(my_address);
        }
    }
}
