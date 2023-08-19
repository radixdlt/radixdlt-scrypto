use scrypto::api::node_modules::royalty::*;
use scrypto::api::*;
use scrypto::prelude::*;

#[blueprint]
mod component_module {
    use crate::ComponentRoyaltyConfig;
    use std::collections::BTreeMap;

    struct ComponentModule {}

    impl ComponentModule {
        pub fn globalize_with_mixed_up_modules() -> ComponentAddress {
            let component = ComponentModule {}.instantiate();

            let rtn = ScryptoVmV1Api.call_function(
                METADATA_MODULE_PACKAGE,
                METADATA_BLUEPRINT,
                METADATA_CREATE_IDENT,
                scrypto_encode(&MetadataCreateInput {}).unwrap(),
            );
            let metadata: Own = scrypto_decode(&rtn).unwrap();

            let rtn = ScryptoVmV1Api.call_function(
                ROYALTY_MODULE_PACKAGE,
                COMPONENT_ROYALTY_BLUEPRINT,
                COMPONENT_ROYALTY_CREATE_IDENT,
                scrypto_encode(&ComponentRoyaltyCreateInput {
                    royalty_config: ComponentRoyaltyConfig::default(),
                })
                .unwrap(),
            );
            let royalty: Own = scrypto_decode(&rtn).unwrap();

            let rtn = ScryptoVmV1Api.call_function(
                ROLE_ASSIGNMENT_MODULE_PACKAGE,
                ROLE_ASSIGNMENT_BLUEPRINT,
                ROLE_ASSIGNMENT_CREATE_IDENT,
                scrypto_encode(&RoleAssignmentCreateInput {
                    owner_role: OwnerRole::None.into(),
                    roles: BTreeMap::new(),
                })
                .unwrap(),
            );
            let role_assignment: Own = scrypto_decode(&rtn).unwrap();

            let address = ScryptoVmV1Api.globalize(
                btreemap!(
                    ObjectModuleId::Main => *component.0.handle().as_node_id(),
                    ObjectModuleId::RoleAssignment => metadata.0,
                    ObjectModuleId::Metadata => royalty.0,
                    ObjectModuleId::Royalty => role_assignment.0,
                ),
                None,
            );

            ComponentAddress::new_or_panic(address.into())
        }
    }
}
