use scrypto::prelude::*;

#[blueprint]
mod mutable_access_rules_component {
    struct MutableAccessRulesComponent {}

    impl MutableAccessRulesComponent {

        pub fn new(roles: Roles) -> Global<MutableAccessRulesComponent> {
            Self {}
                .instantiate()
                .prepare_to_globalize()
                .define_roles(roles)
                .methods(methods! {
                    borrow_funds => ["borrow_funds_auth"];
                    deposit_funds => ["deposit_funds_auth"];
                    set_authority_rules => Public;
                    lock_authority => Public;
                })
                .globalize()
        }

        pub fn new_with_owner(owner_update_access_rule: AccessRule) -> Global<MutableAccessRulesComponent> {
            Self {}
                .instantiate()
                .prepare_to_globalize()
                .define_roles(roles! {
                    "owner" => rule!(require(RADIX_TOKEN)), ["owner_update"];
                    "owner_update" => owner_update_access_rule;
                })
                .methods(methods! {
                    borrow_funds => ["owner"];
                    deposit_funds => ["owner"];
                    set_authority_rules => Public;
                    lock_authority => Public;
                })
                .globalize()
        }

        pub fn access_rules_function(component_address: ComponentAddress) {
            let component: Global<AnyComponent> = component_address.into();
            let _access_rules = component.access_rules();
        }

        pub fn set_authority_rules(&self, authority: String, rule: AccessRule) {
            let access_rules = Runtime::access_rules();
            access_rules.define_role(authority.as_str(), rule);
        }

        pub fn lock_authority(&self, role: String) {
            let access_rules = Runtime::access_rules();
            access_rules.set_role_mutability(role.as_str(), RoleList::none());
        }

        // The methods that the access rules will be added to
        pub fn borrow_funds(&self) {}

        pub fn deposit_funds(&self) {}
    }
}
