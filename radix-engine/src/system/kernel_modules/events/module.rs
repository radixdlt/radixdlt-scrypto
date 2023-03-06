use crate::errors::{ApplicationError, RuntimeError};
use crate::kernel::kernel_api::KernelModuleApi;
use crate::kernel::module::KernelModule;
use crate::system::node::{RENodeInit, RENodeModuleInit};
use crate::system::node_modules::event_schema::PackageEventSchemaSubstate;
use crate::types::*;
use radix_engine_interface::api::types::*;

use super::EventError;

#[derive(Debug, Default, Clone)]
pub struct EventsModule(Vec<(EventTypeIdentifier, Vec<u8>)>);

impl EventsModule {
    pub fn add_event(&mut self, identifier: EventTypeIdentifier, data: Vec<u8>) {
        self.0.push((identifier, data))
    }

    pub fn events(self) -> Vec<(EventTypeIdentifier, Vec<u8>)> {
        self.0
    }
}

impl KernelModule for EventsModule {
    fn before_create_node<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _node_id: &RENodeId,
        _node_init: &RENodeInit,
        node_module_init: &BTreeMap<NodeModuleId, RENodeModuleInit>,
    ) -> Result<(), RuntimeError> {
        // Validating the schema before the node is created.
        if let Some(RENodeModuleInit::PackageEventSchema(PackageEventSchemaSubstate(
            event_schema,
        ))) = node_module_init.get(&NodeModuleId::PackageEventSchema)
        {
            for (_, event_schemas) in event_schema {
                for (_, (local_type_index, schema)) in event_schemas {
                    // Checking that the schema is itself valid
                    schema.validate().map_err(|_| {
                        RuntimeError::ApplicationError(ApplicationError::EventError(
                            EventError::InvalidEventSchema,
                        ))
                    })?;

                    // Ensuring that the event is either a struct or an enum
                    match schema.resolve_type_kind(*local_type_index) {
                        // Structs and Enums are allowed
                        Some(TypeKind::Enum { .. } | TypeKind::Tuple { .. }) => Ok(()),
                        _ => Err(RuntimeError::ApplicationError(
                            ApplicationError::EventError(EventError::InvalidEventSchema),
                        )),
                    }?
                }
            }
        }
        Ok(())
    }
}
