use radix_engine_interface::types::Blueprint;
use radix_engine_interface::*;
use sbor::rust::string::String;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum EventError {
    SchemaNotFoundError {
        blueprint: Blueprint,
        event_name: String,
    },
    EventSchemaNotMatch(String),
    NoAssociatedPackage,
    InvalidActor,
}
