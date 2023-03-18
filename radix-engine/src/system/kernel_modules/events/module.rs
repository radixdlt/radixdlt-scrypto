use crate::errors::{ApplicationError, RuntimeError};
use crate::kernel::kernel_api::KernelModuleApi;
use crate::kernel::module::KernelModule;
use crate::system::node::{RENodeInit, RENodeModuleInit};
use crate::system::node_substates::RuntimeSubstate;
use crate::types::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::blueprints::package::PackageInfoSubstate;
use radix_engine_interface::schema::BlueprintSchema;

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
        _: &mut Y,
        _: &RENodeId,
        node_init: &RENodeInit,
        _: &BTreeMap<NodeModuleId, RENodeModuleInit>,
    ) -> Result<(), RuntimeError> {
        if let RENodeInit::GlobalObject(substates) = node_init {
            if let Some(RuntimeSubstate::PackageInfo(PackageInfoSubstate {
                schema: package_schema,
                ..
            })) = substates.get(&SubstateOffset::Package(PackageOffset::Info))
            {
                for BlueprintSchema {
                    schema,
                    event_schema,
                    ..
                } in package_schema.blueprints.values()
                {
                    // Package schema validation happens when the package is published. No need to redo
                    // it here again.

                    for (expected_event_name, local_type_index) in event_schema.iter() {
                        // Checking that the event name is indeed what the user claims it to be
                        let actual_event_name =
                            schema.resolve_type_metadata(*local_type_index).map_or(
                                Err(RuntimeError::ApplicationError(
                                    ApplicationError::EventError(
                                        EventError::FailedToResolveLocalSchema {
                                            local_type_index: *local_type_index,
                                        },
                                    ),
                                )),
                                |metadata| Ok(metadata.type_name.to_string()),
                            )?;

                        if *expected_event_name != actual_event_name {
                            Err(RuntimeError::ApplicationError(
                                ApplicationError::EventError(EventError::EventNameMismatch {
                                    expected: expected_event_name.to_string(),
                                    actual: actual_event_name,
                                }),
                            ))?
                        }

                        // Checking that the event is either a struct or an enum
                        let type_kind = schema.resolve_type_kind(*local_type_index).map_or(
                            Err(RuntimeError::ApplicationError(
                                ApplicationError::EventError(
                                    EventError::FailedToResolveLocalSchema {
                                        local_type_index: *local_type_index,
                                    },
                                ),
                            )),
                            Ok,
                        )?;
                        match type_kind {
                            // Structs and Enums are allowed
                            TypeKind::Enum { .. } | TypeKind::Tuple { .. } => Ok(()),
                            _ => Err(RuntimeError::ApplicationError(
                                ApplicationError::EventError(EventError::InvalidEventSchema),
                            )),
                        }?;
                    }
                }
            }
        }

        Ok(())
    }
}
