use scrypto::prelude::*;

blueprint! {
    struct MutableAccessRulesComponent {}

    impl MutableAccessRulesComponent {
        pub fn new(access_rules: Vec<AccessRules>) -> ComponentAddress {
            let mut component = Self {}.instantiate();
            for access_rule in access_rules {
                component.add_access_check(access_rule);
            }
            component.globalize()
        }

        pub fn access_rules_function(component_address: ComponentAddress) -> Vec<AccessRules> {
            let component = borrow_component!(component_address);
            component
                .access_rules()
                .into_iter()
                .map(|x| x.access_rules())
                .collect()
        }

        pub fn access_rules_method(&self) -> Vec<AccessRules> {
            let component = Component(Runtime::actor().as_component().0);
            component
                .access_rules()
                .into_iter()
                .map(|x| x.access_rules())
                .collect()
        }

        pub fn set_method_auth(&self, index: usize, method_name: String, rule: AccessRule) {
            let component = Component(Runtime::actor().as_component().0);
            component
                .access_rules()
                .get_mut(index)
                .unwrap()
                .set_method_auth(&method_name, rule);
        }

        pub fn set_default_auth(&self, index: usize, rule: AccessRule) {
            let component = Component(Runtime::actor().as_component().0);
            component
                .access_rules()
                .get_mut(index)
                .unwrap()
                .set_default(rule);
        }

        pub fn lock_method_auth(&self, index: usize, method_name: String) {
            let component = Component(Runtime::actor().as_component().0);
            component
                .access_rules()
                .get_mut(index)
                .unwrap()
                .lock_method_auth(&method_name);
        }

        pub fn lock_default_auth(&self, index: usize) {
            let component = Component(Runtime::actor().as_component().0);
            component
                .access_rules()
                .get_mut(index)
                .unwrap()
                .lock_default();
        }

        // The methods that the access rules will be added to
        pub fn borrow_funds(&self) {}
        pub fn deposit_funds(&self) {}
    }
}
