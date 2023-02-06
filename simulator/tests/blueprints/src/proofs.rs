use scrypto::prelude::*;

#[blueprint]
mod proofs {
    struct Proofs {}

    impl Proofs {
        pub fn new() -> (ComponentAddress, Vec<Bucket>) {
            // Creating three badges: admin badge, supervisor badge, super admin badge
            let supervisor_badge = Self::new_badge("supervisor badge");
            let admin_badge = Self::new_badge("admin badge");
            let superadmin_badge = Self::new_badge("superadmin badge");

            // Creating a token which can only be withdrawn and minted when all three of the above badges are present
            let organizational_access_rule: AccessRule = rule!(
                require(supervisor_badge.resource_address())
                    && require(admin_badge.resource_address())
                    && require(superadmin_badge.resource_address())
            );
            let token = ResourceBuilder::new_fungible()
                .mintable(organizational_access_rule.clone(), LOCKED)
                .restrict_withdraw(organizational_access_rule.clone(), LOCKED)
                .initial_supply(100);

            // Creating the access rules for the component and instantiating an new component
            let access_rules: AccessRules = AccessRules::new()
                .method(
                    "organizational_authenticated_method",
                    organizational_access_rule,
                    LOCKED,
                )
                .default(rule!(deny_all), rule!(deny_all));

            let mut local_component = Self {}.instantiate();
            local_component.add_access_check(access_rules);

            (
                local_component.globalize(),
                vec![supervisor_badge, admin_badge, superadmin_badge, token],
            )
        }

        fn new_badge(name: &str) -> Bucket {
            ResourceBuilder::new_fungible()
                .divisibility(0)
                .metadata("name", name)
                .initial_supply(1)
        }

        pub fn organizational_authenticated_method(&self) {
            info!("We are inside the authenticated method");
        }
    }
}
