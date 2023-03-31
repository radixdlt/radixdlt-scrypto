use scrypto::prelude::*;

#[blueprint]
mod deep_auth_rules_on_create {
    struct DeepAuthRulesOnCreate {}

    impl DeepAuthRulesOnCreate {
        pub fn new(
            resource_address: ResourceAddress,
            access_rules_depth: usize,
        ) -> ComponentAddress {
            let component = Self {}.instantiate();
            component.globalize_with_access_rules(generate_deep_access_rules(
                resource_address,
                access_rules_depth,
            ))
        }
    }
}

fn generate_deep_access_rules(
    resource_address: ResourceAddress,
    exceed_depth: usize,
) -> AccessRulesConfig {
    let mut access_rule_node = AccessRuleNode::ProofRule(ProofRule::Require(
        SoftResourceOrNonFungible::StaticResource(resource_address),
    ));
    let mut curr_depth = 6; // The inner bit and the outer mapping
    while curr_depth < exceed_depth {
        access_rule_node = AccessRuleNode::AllOf(vec![access_rule_node]);
        curr_depth += 2;
    }
    AccessRulesConfig::new().default(AccessRule::Protected(access_rule_node), AccessRule::DenyAll)
}
