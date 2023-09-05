use scrypto::prelude::*;

#[blueprint]
mod mutable_role_assignment_component {
    enable_method_auth! {
        roles {
            borrow_funds_auth => updatable_by: [deposit_funds_auth];
            deposit_funds_auth => updatable_by: [];
        },
        methods {
            borrow_funds => restrict_to: [borrow_funds_auth];
            deposit_funds => restrict_to: [deposit_funds_auth];
            set_authority_rules => PUBLIC;
        }
    }

    struct MutableRulesComponent {}

    impl MutableRulesComponent {
        pub fn new(roles: RoleAssignmentInit) -> Global<MutableRulesComponent> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .roles(roles)
                .globalize()
        }

        pub fn new_with_owner(owner_role: OwnerRole) -> Global<MutableRulesComponent> {
            Self {}
                .instantiate()
                .prepare_to_globalize(owner_role)
                .roles(roles! {
                    borrow_funds_auth => rule!(require(XRD));
                    deposit_funds_auth => OWNER;
                })
                .globalize()
        }

        pub fn set_authority_rules(&self, role: String, rule: Rule) {
            Runtime::global_component().set_role(role.as_str(), rule);
        }

        // The methods that the access rules will be added to
        pub fn borrow_funds(&self) {}

        pub fn deposit_funds(&self) {}
    }
}
