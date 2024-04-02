use crate::internal_prelude::*;
use crate::system::system_db_reader::{SystemDatabaseReader, SystemReaderError};
use crate::system::system_type_checker::{BlueprintTypeTarget, SchemaValidationMeta};
use radix_common::constants::BLUEPRINT_PAYLOAD_MAX_DEPTH;
use radix_engine_interface::blueprints::package::{BlueprintPayloadIdentifier, BlueprintVersion};
use radix_engine_interface::types::Emitter;
use radix_substate_store_interface::interface::SubstateDatabase;

pub trait ApplicationEventChecker: Default {
    type ApplicationEventCheckerResults: Debug + Default;

    fn on_event(&mut self, _info: BlueprintInfo, _event_id: EventTypeIdentifier, _event: &Vec<u8>) {
    }

    fn on_finish(&self) -> Self::ApplicationEventCheckerResults {
        Self::ApplicationEventCheckerResults::default()
    }
}

impl ApplicationEventChecker for () {
    type ApplicationEventCheckerResults = ();
}

#[derive(Debug)]
pub enum SystemEventCheckerError {
    MissingObjectTypeTarget,
    MissingPayloadSchema(SystemReaderError),
    InvalidEvent,
}

pub struct SystemEventChecker<A: ApplicationEventChecker> {
    application_checker: A,
}

impl<A: ApplicationEventChecker> SystemEventChecker<A> {
    pub fn new() -> Self {
        Self {
            application_checker: A::default(),
        }
    }

    pub fn check_all_events<S: SubstateDatabase>(
        &mut self,
        substate_db: &S,
        events: &Vec<Vec<(EventTypeIdentifier, Vec<u8>)>>,
    ) -> Result<A::ApplicationEventCheckerResults, SystemEventCheckerError> {
        let reader = SystemDatabaseReader::new(substate_db);

        for (event_id, event_payload) in events.iter().flatten() {
            let type_target = match &event_id.0 {
                Emitter::Method(node_id, module_id) => reader
                    .get_blueprint_type_target(node_id, *module_id)
                    .map_err(|_| SystemEventCheckerError::MissingObjectTypeTarget)?,
                Emitter::Function(blueprint_id) => BlueprintTypeTarget {
                    blueprint_info: BlueprintInfo {
                        blueprint_id: blueprint_id.clone(),
                        blueprint_version: BlueprintVersion::default(),
                        outer_obj_info: OuterObjectInfo::None,
                        features: indexset!(),
                        generic_substitutions: vec![],
                    },
                    meta: SchemaValidationMeta::Blueprint,
                },
            };

            let event_schema = reader
                .get_blueprint_payload_schema(
                    &type_target,
                    &BlueprintPayloadIdentifier::Event(event_id.1.clone()),
                )
                .map_err(SystemEventCheckerError::MissingPayloadSchema)?;

            reader
                .validate_payload(&event_payload, &event_schema, BLUEPRINT_PAYLOAD_MAX_DEPTH)
                .map_err(|_| SystemEventCheckerError::InvalidEvent)?;

            self.application_checker.on_event(
                type_target.blueprint_info,
                event_id.clone(),
                event_payload,
            );
        }

        let results = self.application_checker.on_finish();

        Ok(results)
    }
}
