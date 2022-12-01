use scrypto::prelude::*;

blueprint! {
    struct MutableAccessRulesComponent {}

    impl MutableAccessRulesComponent {
        pub fn new(access_rules: Vec<AccessRules>) -> ComponentAddress {
            let mut component = Self {}.instantiate();
            for access_rule in access_rules {
                component.add_access_check(access_rule);
            }
            component.globalize_no_owner()
        }

        pub fn access_rules_function(
            component_address: ComponentAddress,
        ) -> Vec<ComponentAccessRules> {
            let component = borrow_component!(component_address);
            component.access_rules_chain()
        }

        pub fn access_rules_method(&self) -> Vec<ComponentAccessRules> {
            let component = Component(Runtime::actor().as_component().0);
            component.access_rules_chain()
        }

        pub fn set_method_auth(&self, index: usize, method_name: String, rule: AccessRule) {
            let component = Component(Runtime::actor().as_component().0);
            component
                .access_rules_chain()
                .get_mut(index)
                .unwrap()
                .set_method_auth(&method_name, rule);
        }

        pub fn lock_method_auth(&self, index: usize, method_name: String) {
            let component = Component(Runtime::actor().as_component().0);
            component
                .access_rules_chain()
                .get_mut(index)
                .unwrap()
                .lock_method_auth(&method_name);
        }

        // The methods that the access rules will be added to
        pub fn borrow_funds(&self) {}
        pub fn deposit_funds(&self) {}
    }
}
