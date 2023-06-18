use scrypto::prelude::*;

#[blueprint]
mod mutable_access_rules_component {
    enable_method_auth! {
        roles {
            borrow_funds_auth,
            deposit_funds_auth
        },
        methods {
            borrow_funds => borrow_funds_auth;
            deposit_funds => deposit_funds_auth;
            set_authority_rules => PUBLIC;
            lock_authority => PUBLIC;
        }
    }

    struct MutableAccessRulesComponent {}

    impl MutableAccessRulesComponent {
        pub fn new(roles: Roles) -> Global<MutableAccessRulesComponent> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .roles(roles)
                .globalize()
        }

        pub fn new_with_owner(
            owner_update_access_rule: AccessRule,
        ) -> Global<MutableAccessRulesComponent> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .roles(roles! {
                    borrow_funds_auth => rule!(require(RADIX_TOKEN)), updaters: deposit_funds_auth;
                    deposit_funds_auth => owner_update_access_rule;
                })
                .globalize()
        }

        pub fn access_rules_function(component_address: ComponentAddress) {
            let component: Global<AnyComponent> = component_address.into();
            let _access_rules = component.access_rules();
        }

        pub fn set_authority_rules(&self, authority: String, rule: AccessRule) {
            let access_rules = Runtime::access_rules();
            access_rules.update_role_rule(authority.as_str(), rule);
        }

        pub fn lock_authority(&self, role: String) {
            let access_rules = Runtime::access_rules();
            access_rules.freeze_role(role.as_str());
        }

        // The methods that the access rules will be added to
        pub fn borrow_funds(&self) {}

        pub fn deposit_funds(&self) {}
    }
}
