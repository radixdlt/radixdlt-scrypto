use scrypto::prelude::*;

#[blueprint]
mod royalty_edge_cases {
    enable_package_royalties! {
        // We manipulate the value of this manually in tests by modifying the [`PackageDefinition`]
        instantiate => Free;
        instantiate_with_missing_method_royalty => Free;
        func => Free;
        method => Free;
    }

    struct RoyaltyEdgeCases;

    impl RoyaltyEdgeCases {
        pub fn instantiate(royalty_amount: RoyaltyAmount) -> Global<RoyaltyEdgeCases> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::Updatable(AccessRule::AllowAll))
                .enable_component_royalties(component_royalties! {
                    init {
                        method => royalty_amount, updatable;
                    }
                })
                .globalize()
        }

        pub fn instantiate_with_missing_method_royalty() -> Global<RoyaltyEdgeCases> {
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
            let royalty_config: Option<ModuleConfig<ComponentRoyaltyConfig>> = {
                Some(ModuleConfig {
                    init: ComponentRoyaltyConfig {
                        royalty_amounts: indexmap! {}, /* Intentionally no royalty for `method`. */
                    },
                    roles: RoleAssignmentInit::new(),
                })
            };
            if let Some(royalty_config) = royalty_config {
                roles.insert(ModuleId::Royalty, royalty_config.roles);
                let royalty = Royalty::new(royalty_config.init);
                modules.insert(AttachedModuleId::Royalty, *royalty.handle().as_node_id());
            }

            // Role Assignment
            {
                let role_assignment = RoleAssignment::new(OwnerRole::None, roles);
                modules.insert(
                    AttachedModuleId::RoleAssignment,
                    *role_assignment.handle().as_node_id(),
                );
            }

            let address =
                ScryptoVmV1Api::object_globalize(*this.handle().as_node_id(), modules, None);

            Global(
                <royalty_edge_cases::RoyaltyEdgeCases as scrypto::component::HasStub>::Stub::new(
                    ObjectStubHandle::Global(address),
                ),
            )
        }

        pub fn func() {}
        pub fn method(&self) {}
    }
}
