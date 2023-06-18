use radix_engine_interface::types::BlueprintId;
use radix_engine_interface::*;
use sbor::rust::string::String;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum EventError {
    SchemaNotFoundError {
        blueprint: BlueprintId,
        event_name: String,
    },
    EventSchemaNotMatch(String),
    NoAssociatedPackage,
    InvalidActor,
}
