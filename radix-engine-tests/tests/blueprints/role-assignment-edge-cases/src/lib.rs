use scrypto::prelude::*;

#[blueprint]
mod role_assignment_edge_cases {
    enable_package_royalties! {
        instantiate => Free;
    }

    struct RoleAssignmentEdgeCases;

    impl RoleAssignmentEdgeCases {
        pub fn instantiate(
            init_roles: IndexMap<ModuleId, RoleAssignmentInit>,
            set_roles: IndexMap<(ModuleId, String), AccessRule>,
        ) -> Global<RoleAssignmentEdgeCases> {
            let this = Self {}.instantiate();

            let mut modules = index_map_new();
            let mut roles = init_roles;

            // Main
            {
                roles.entry(ModuleId::Main).or_default();
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

                for ((module_id, role_name), rule) in set_roles.into_iter() {
                    match module_id {
                        ModuleId::Main => &role_assignment.set_role(role_name.as_str(), rule),
                        ModuleId::Royalty => {
                            &role_assignment.set_component_royalties_role(role_name.as_str(), rule)
                        }
                        ModuleId::RoleAssignment => {
                            &role_assignment.set_role_assignment_role(role_name.as_str(), rule)
                        }
                        ModuleId::Metadata => {
                            &role_assignment.set_metadata_role(role_name.as_str(), rule)
                        }
                    };
                }

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
