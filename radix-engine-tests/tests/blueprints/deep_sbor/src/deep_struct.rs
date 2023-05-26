use scrypto::prelude::*;

#[blueprint]
mod deep_struct {
    struct DeepStruct {
        deep_object: Option<AuthorityRules>,
    }

    impl DeepStruct {
        pub fn new() -> Global<DeepStruct> {
            Self { deep_object: None }.instantiate().globalize()
        }

        pub fn set_depth(&mut self, resource_address: ResourceAddress, exceed_depth: usize) {
            self.deep_object = Some(generate_deep_access_rules(resource_address, exceed_depth));
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
    authority_rules.set_main_authority_rule(
        "test",
        AccessRule::Protected(access_rule_node.clone()),
        AccessRule::Protected(access_rule_node),
    );
    authority_rules
}
