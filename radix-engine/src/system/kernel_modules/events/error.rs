use radix_engine_interface::crypto::Hash;
use radix_engine_interface::data::scrypto::model::PackageAddress;
use radix_engine_interface::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum EventError {
    SchemaNotFoundError {
        package_address: PackageAddress,
        blueprint_name: String,
        schema_hash: Hash,
    },
    InvalidEventSchema,
    NoAssociatedPackage,
    FailedToSborEncodeEventSchema,
    FailedToSborEncodeEvent,
    InvalidActor,
}
