use scrypto::prelude::*;

#[blueprint]
mod deep_auth_rules_on_create {
    struct DeepAuthRulesOnCreate {}

    impl DeepAuthRulesOnCreate {
        pub fn new(
            resource_address: ResourceAddress,
            role_assignment_depth: usize,
        ) -> Global<DeepAuthRulesOnCreate> {
            let component = Self {}.instantiate();
            let roles = generate_deep_access_rules(resource_address, role_assignment_depth);
            component
                .prepare_to_globalize(OwnerRole::None)
                .roles(roles)
                .globalize()
        }
    }
}

fn generate_deep_access_rules(
    resource_address: ResourceAddress,
    exceed_depth: usize,
) -> RoleAssignmentInit {
    let mut composite_requirement = CompositeRequirement::BasicRequirement(
        BasicRequirement::Require(ResourceOrNonFungible::Resource(resource_address)),
    );
    let mut curr_depth = 6; // The inner bit and the outer mapping
    while curr_depth < exceed_depth {
        composite_requirement = CompositeRequirement::AllOf(vec![composite_requirement]);
        curr_depth += 2;
    }

    roles2! {
        "test" => AccessRule::Protected(composite_requirement.clone()), updatable;
    }
}
