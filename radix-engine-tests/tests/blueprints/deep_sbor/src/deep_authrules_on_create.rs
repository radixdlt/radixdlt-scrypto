use scrypto::prelude::*;

#[blueprint]
mod deep_auth_rules_on_create {
    struct DeepAuthRulesOnCreate {}

    impl DeepAuthRulesOnCreate {
        pub fn new(
            resource_address: ResourceAddress,
            access_rules_depth: usize,
        ) -> Global<DeepAuthRulesOnCreateComponent> {
            let component = Self {}.instantiate();
            let access_rules = AccessRules::new(
                MethodAuthorities::new(),
                generate_deep_access_rules(resource_address, access_rules_depth),
            );
            component.attach_access_rules(access_rules).globalize()
        }
    }
}

fn generate_deep_access_rules(
    resource_address: ResourceAddress,
    exceed_depth: usize,
) -> AuthorityRules {
    let mut access_rule_node = AccessRuleNode::ProofRule(ProofRule::Require(
        ResourceOrNonFungible::Resource(resource_address),
    ));
    let mut curr_depth = 6; // The inner bit and the outer mapping
    while curr_depth < exceed_depth {
        access_rule_node = AccessRuleNode::AllOf(vec![access_rule_node]);
        curr_depth += 2;
    }
    let mut authority_rules = AuthorityRules::new();
    authority_rules.set_rule(
        "test",
        AccessRule::Protected(access_rule_node.clone()),
        AccessRule::Protected(access_rule_node),
    );
    authority_rules
}
