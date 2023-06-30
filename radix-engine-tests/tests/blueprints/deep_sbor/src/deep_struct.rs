use scrypto::prelude::*;

#[blueprint]
mod deep_struct {
    struct DeepStruct {
        deep_object: Option<RolesInit>,
    }

    impl DeepStruct {
        pub fn new() -> Global<DeepStruct> {
            Self { deep_object: None }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn set_depth(&mut self, resource_address: ResourceAddress, exceed_depth: usize) {
            self.deep_object = Some(generate_deep_access_rules(resource_address, exceed_depth));
        }
    }
}

fn generate_deep_access_rules(resource_address: ResourceAddress, exceed_depth: usize) -> RolesInit {
    let mut access_rule_node = AccessRuleNode::ProofRule(ProofRule::Require(
        ResourceOrNonFungible::Resource(resource_address),
    ));
    let mut curr_depth = 6; // The inner bit and the outer mapping
    while curr_depth < exceed_depth {
        access_rule_node = AccessRuleNode::AllOf(vec![access_rule_node]);
        curr_depth += 2;
    }

    roles2! {
        "test" => AccessRule::Protected(access_rule_node.clone()), updatable;
    }
}
