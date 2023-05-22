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
                .method_permissions(MethodPermissions {
                    borrow_funds: MethodPermission::Protected(["borrow_funds_auth"].into()),
                    deposit_funds: MethodPermission::Protected(["deposit_funds_auth"].into()),
                    set_authority_rules: MethodPermission::Public,
                    lock_authority: MethodPermission::Public,
                })
                .globalize()
        }

        pub fn access_rules_function(component_address: ComponentAddress) {
            let component: Global<AnyComponent> = component_address.into();
            let _access_rules = component.access_rules();
        }

        pub fn set_authority_rules(&self, authority: String, rule: AccessRule) {
            let access_rules = Runtime::access_rules();
            access_rules.set_authority_rule(authority.as_str(), rule);
        }

        pub fn lock_authority(&self, authority: String) {
            let access_rules = Runtime::access_rules();
            access_rules.set_authority_mutability(authority.as_str(), vec![]);
        }

        // The methods that the access rules will be added to
        pub fn borrow_funds(&self) {}

        pub fn deposit_funds(&self) {}
    }
}
