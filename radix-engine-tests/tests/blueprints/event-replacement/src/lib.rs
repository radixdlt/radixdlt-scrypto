use scrypto::prelude::*;

#[blueprint]
mod event_replacement {
    struct EventReplacement;

    impl EventReplacement {
        pub fn instantiate() -> Global<EventReplacement> {
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

                /*
                Event replacements are required when a module emits an event before becoming
                attached. So, in here, we're updating the metadata so that the metadata module emits
                an event prior to attachment. We want to ensure that the final events in the receipt
                show that this event has been emitted by the component's metadata module.
                */

                metadata.set("Hello", "World".to_owned());
            };

            // Royalties
            let royalty_config: Option<ModuleConfig<ComponentRoyaltyConfig>> = {
                Some(ModuleConfig {
                    init: ComponentRoyaltyConfig {
                        royalty_amounts: indexmap! {
                            "instantiate".to_owned() => (RoyaltyAmount::Free, true)
                        },
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
                <event_replacement::EventReplacement as scrypto::component::HasStub>::Stub::new(
                    ObjectStubHandle::Global(address),
                ),
            )
        }

        pub fn func() {}
        pub fn method(&self) {}
    }
}
