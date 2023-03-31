use scrypto::prelude::*;

#[blueprint]
mod mutable_access_rules_component {
    struct MutableAccessRulesComponent {}

    impl MutableAccessRulesComponent {
        pub fn new(access_rules: AccessRulesConfig) -> ComponentAddress {
            let component = Self {}.instantiate();
            component.globalize_with_access_rules(access_rules)
        }

        pub fn access_rules_function(component_address: ComponentAddress) -> AttachedAccessRules {
            let component = borrow_component!(component_address);
            component.access_rules()
        }

        pub fn access_rules_method(&self) -> Vec<AttachedAccessRules> {
            todo!("Support for self");
        }

        pub fn set_method_auth(&self, _index: usize, _method_name: String, _rule: AccessRule) {
            todo!("Support for self mutable auth");
        }

        pub fn lock_method_auth(&self, _index: usize, _method_name: String) {
            todo!("Support for self mutable auth");
        }

        // The methods that the access rules will be added to
        pub fn borrow_funds(&self) {}
        pub fn deposit_funds(&self) {}
    }
}
