use scrypto::prelude::*;

#[blueprint]
mod deep_struct {
    struct DeepStruct {
        deep_object: Option<RoleAssignmentInit>,
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
