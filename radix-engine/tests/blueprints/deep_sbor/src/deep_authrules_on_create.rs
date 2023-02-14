use scrypto::prelude::*;

#[blueprint]
mod deep_auth_rules_on_create {
    struct DeepAuthRulesOnCreate {}

    impl DeepAuthRulesOnCreate {
        pub fn new(resource_address: ResourceAddress, access_rules_depth: u8) -> ComponentAddress {
            let mut component = Self {}.instantiate();

            component.add_access_check(generate_deep_access_rules(
                resource_address,
                access_rules_depth,
            ));

            component.globalize()
        }
    }
}

fn generate_deep_access_rules(resource_address: ResourceAddress, exceed_depth: u8) -> AccessRules {
    let mut access_rule_node = AccessRuleNode::ProofRule(ProofRule::Require(
        SoftResourceOrNonFungible::StaticResource(resource_address),
    ));
    let mut curr_depth = 6; // The inner bit and the outer mapping
    while curr_depth < exceed_depth {
        access_rule_node = AccessRuleNode::AllOf(vec![access_rule_node]);
        curr_depth += 2;
    }
    AccessRules::new().default(AccessRule::Protected(access_rule_node), AccessRule::DenyAll)
}
