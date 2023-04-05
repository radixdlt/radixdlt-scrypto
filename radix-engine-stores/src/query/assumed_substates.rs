use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::schema::KeyValueStoreSchema;
use radix_engine_interface::types::{ObjectInfo, ResourceAddress};
use radix_engine_interface::*;

// TODO: de-dup

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct VaultInfoSubstate {
    pub resource_address: ResourceAddress,
    pub resource_type: ResourceType,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum TypeInfoSubstate {
    Object(ObjectInfo),
    KeyValueStore(KeyValueStoreSchema),
}
