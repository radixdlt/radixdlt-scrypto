use radix_engine_common::types::*;
use sbor::rust::prelude::*;
use scrypto_schema::KeyValueStoreSchema;

// TODO: Add locked entry interface rather than using substate api
pub trait ClientKeyValueStoreApi<E> {
    /// Creates a new key value store with a given schema
    fn new_key_value_store(&mut self, schema: KeyValueStoreSchema) -> Result<NodeId, E>;

    /// Get info regarding a visible key value store
    fn get_key_value_store_info(&mut self, node_id: &NodeId) -> Result<KeyValueStoreSchema, E>;
}
