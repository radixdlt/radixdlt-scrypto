use scrypto::prelude::*;

#[blueprint]
mod mutable_access_rules_component {
    enable_method_auth! {
        roles {
            borrow_funds_auth => updatable_by: [deposit_funds_auth];
            deposit_funds_auth => updatable_by: [];
        },
        methods {
            borrow_funds => restrict_to: [borrow_funds_auth];
            deposit_funds => restrict_to: [deposit_funds_auth];
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
                    borrow_funds_auth => rule!(require(RADIX_TOKEN)), updatable;
                    deposit_funds_auth => owner_update_access_rule, locked;
                })
                .globalize()
        }

        pub fn set_authority_rules(&self, role: String, rule: AccessRule) {
            let access_rules = Runtime::access_rules();
            access_rules.set_role(role.as_str(), rule);
        }

        pub fn lock_authority(&self, role: String) {
            let access_rules = Runtime::access_rules();
            access_rules.lock_role(role.as_str());
        }

        // The methods that the access rules will be added to
        pub fn borrow_funds(&self) {}

        pub fn deposit_funds(&self) {}
    }
}
