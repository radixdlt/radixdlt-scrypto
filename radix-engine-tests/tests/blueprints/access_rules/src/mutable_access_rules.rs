use scrypto::prelude::*;

#[blueprint]
mod mutable_access_rules_component {
    struct MutableAccessRulesComponent {}

    impl MutableAccessRulesComponent {
        pub fn new(
            method_authorities: MethodAuthorities,
            authority_rules: AuthorityRules,
        ) -> ComponentAddress {
            let component = Self {}.instantiate();
            component.globalize_with_access_rules(method_authorities, authority_rules)
        }

        pub fn access_rules_function(component_address: ComponentAddress) -> AttachedAccessRules {
            let component = borrow_component!(component_address);
            component.access_rules()
        }

        pub fn set_authority_rule(&self, authority: String, rule: AccessRule) {
            let access_rules = Runtime::get_access_rules();
            access_rules.set_authority_rule(authority.as_str(), rule);
        }

        pub fn lock_authority(&self, authority: String) {
            let access_rules = Runtime::get_access_rules();
            access_rules.set_authority_mutability(authority.as_str(), AccessRule::DenyAll);
        }

        // The methods that the access rules will be added to
        pub fn borrow_funds(&self) {}
        pub fn deposit_funds(&self) {}
    }
}
