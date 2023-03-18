use radix_engine_interface::data::scrypto::model::PackageAddress;
use radix_engine_interface::*;
use sbor::rust::string::String;
use sbor::LocalTypeIndex;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum EventError {
    SchemaNotFoundError {
        package_address: PackageAddress,
        blueprint_name: String,
        event_name: String,
    },
    FailedToResolveLocalSchema {
        local_type_index: LocalTypeIndex,
    },
    EventNameMismatch {
        expected: String,
        actual: String,
    },
    InvalidEventSchema,
    EventSchemaNotMatch(String),
    NoAssociatedPackage,
    InvalidActor,
}
