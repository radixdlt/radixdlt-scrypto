use scrypto::prelude::*;

#[blueprint]
mod role_assignment_edge_cases {
    enable_package_royalties! {
        instantiate => Free;
    }

    struct RoleAssignmentEdgeCases;

    impl RoleAssignmentEdgeCases {
        pub fn instantiate() -> Global<RoleAssignmentEdgeCases> {
            let this = Self {}.instantiate();

            let mut modules = index_map_new();
            let mut roles = index_map_new();

            // Main
            {
                roles.insert(ModuleId::Main, Default::default());
            }

            // Metadata
            {
                let metadata_config = ModuleConfig::<MetadataInit>::default();
                let metadata = Metadata::new_with_data(metadata_config.init);
                modules.insert(AttachedModuleId::Metadata, *metadata.handle().as_node_id());
                roles.insert(ModuleId::Metadata, metadata_config.roles);
            };

            // Royalties
            let royalty_config: Option<ModuleConfig<ComponentRoyaltyConfig>> = None;
            if let Some(royalty_config) = royalty_config {
                roles.insert(ModuleId::Royalty, royalty_config.roles);
                let royalty = Royalty::new(royalty_config.init);
                modules.insert(AttachedModuleId::Royalty, *royalty.handle().as_node_id());
            }

            // Role Assignment
            {
                let role_assignment = RoleAssignment::new(OwnerRole::None, roles);

                role_assignment.set_role_assignment_role(OWNER_ROLE, rule!(deny_all));
                role_assignment.set_role_assignment_role(SELF_ROLE, rule!(deny_all));

                modules.insert(
                    AttachedModuleId::RoleAssignment,
                    *role_assignment.handle().as_node_id(),
                );
            }

            let address =
                ScryptoVmV1Api::object_globalize(*this.handle().as_node_id(), modules, None);

            Global(
                <role_assignment_edge_cases::RoleAssignmentEdgeCases as scrypto::component::HasStub>::Stub::new(
                    ObjectStubHandle::Global(address),
                ),
            )
        }
    }
}
