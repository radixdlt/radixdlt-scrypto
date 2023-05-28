use scrypto::prelude::*;

#[blueprint]
mod mutable_access_rules_component {
    define_permissions! {
        borrow_funds => ["borrow_funds_auth"];
        deposit_funds => ["deposit_funds_auth"];
        set_authority_rules => Public;
        lock_authority => Public;
    }

    struct MutableAccessRulesComponent {}

    impl MutableAccessRulesComponent {
        pub fn new(roles: Roles) -> Global<MutableAccessRulesComponent> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .define_roles(roles)
                .globalize()
        }

        pub fn new_with_owner(
            owner_update_access_rule: AccessRule,
        ) -> Global<MutableAccessRulesComponent> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .define_roles(roles! {
                    "borrow_funds_auth" => rule!(require(RADIX_TOKEN)), mut ["deposit_funds_auth"];
                    "deposit_funds_auth" => owner_update_access_rule;
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
            access_rules.update_role_mutability(role.as_str(), RoleList::none());
        }

        // The methods that the access rules will be added to
        pub fn borrow_funds(&self) {}

        pub fn deposit_funds(&self) {}
    }
}
