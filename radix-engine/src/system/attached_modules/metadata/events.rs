use crate::internal_prelude::*;
use radix_engine_interface::object_modules::metadata::MetadataValue;

#[derive(ScryptoSbor, ScryptoEvent, Debug, PartialEq, Eq)]
pub struct SetMetadataEvent {
    pub key: String,
    pub value: MetadataValue,
}

#[derive(ScryptoSbor, ScryptoEvent, Debug)]
pub struct RemoveMetadataEvent {
    pub key: String,
}
