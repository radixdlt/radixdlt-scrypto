use scrypto::object_modules::royalty::*;
use scrypto::prelude::*;

#[blueprint]
mod component_module {
    struct ComponentModule {}

    impl ComponentModule {
        pub fn globalize_with_mixed_up_modules() -> ComponentAddress {
            let component = ComponentModule {}.instantiate();

            let rtn = ScryptoVmV1Api::blueprint_call(
                METADATA_MODULE_PACKAGE,
                METADATA_BLUEPRINT,
                METADATA_CREATE_IDENT,
                scrypto_encode(&MetadataCreateInput {}).unwrap(),
            );
            let metadata: Own = scrypto_decode(&rtn).unwrap();

            let rtn = ScryptoVmV1Api::blueprint_call(
                ROYALTY_MODULE_PACKAGE,
                COMPONENT_ROYALTY_BLUEPRINT,
                COMPONENT_ROYALTY_CREATE_IDENT,
                scrypto_encode(&ComponentRoyaltyCreateInput {
                    royalty_config: ComponentRoyaltyConfig::default(),
                })
                .unwrap(),
            );
            let royalty: Own = scrypto_decode(&rtn).unwrap();

            let rtn = ScryptoVmV1Api::blueprint_call(
                ROLE_ASSIGNMENT_MODULE_PACKAGE,
                ROLE_ASSIGNMENT_BLUEPRINT,
                ROLE_ASSIGNMENT_CREATE_IDENT,
                scrypto_encode(&RoleAssignmentCreateInput {
                    owner_role: OwnerRole::None.into(),
                    roles: index_map_new(),
                })
                .unwrap(),
            );
            let role_assignment: Own = scrypto_decode(&rtn).unwrap();

            let address = ScryptoVmV1Api::object_globalize(
                *component.0.handle().as_node_id(),
                indexmap!(
                    AttachedModuleId::RoleAssignment => metadata.0,
                    AttachedModuleId::Metadata => royalty.0,
                    AttachedModuleId::Royalty => role_assignment.0,
                ),
                None,
            );

            ComponentAddress::new_or_panic(address.into())
        }
    }
}
