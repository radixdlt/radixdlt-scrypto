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
        pub fn new(roles: RolesInit) -> Global<MutableAccessRulesComponent> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .roles(roles)
                .globalize()
        }

        pub fn new_with_owner(owner_role: OwnerRole) -> Global<MutableAccessRulesComponent> {
            Self {}
                .instantiate()
                .prepare_to_globalize(owner_role)
                .roles(roles! {
                    borrow_funds_auth => rule!(require(RADIX_TOKEN)), updatable;
                    deposit_funds_auth => OWNER, locked;
                })
                .globalize()
        }

        pub fn set_authority_rules(&self, role: String, rule: AccessRule) {
            Runtime::global_component().set_role(role.as_str(), rule);
        }

        pub fn lock_authority(&self, role: String) {
            Runtime::global_component().lock_role(role.as_str());
        }

        // The methods that the access rules will be added to
        pub fn borrow_funds(&self) {}

        pub fn deposit_funds(&self) {}
    }
}
