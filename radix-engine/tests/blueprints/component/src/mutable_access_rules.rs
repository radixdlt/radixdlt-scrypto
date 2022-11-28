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

        // The methods that the access rules will be added to
        pub fn borrow_funds() {}
        pub fn deposit_funds() {}
    }
}
