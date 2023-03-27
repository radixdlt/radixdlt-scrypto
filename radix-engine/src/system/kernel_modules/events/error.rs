use radix_engine_interface::data::scrypto::model::PackageAddress;
use radix_engine_interface::*;
use sbor::rust::string::String;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum EventError {
    SchemaNotFoundError {
        package_address: PackageAddress,
        blueprint_name: String,
        event_name: String,
    },
    EventSchemaNotMatch(String),
    NoAssociatedPackage,
    InvalidActor,
}
