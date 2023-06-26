use radix_engine_common::types::*;
use radix_engine_interface::api::key_value_entry_api::KeyValueEntryHandle;
use radix_engine_interface::api::LockFlags;
use sbor::rust::prelude::*;
use scrypto_schema::KeyValueStoreSchema;

pub trait ClientKeyValueStoreApi<E> {
    /// Creates a new key value store with a given schema
    fn key_value_store_new(&mut self, schema: KeyValueStoreSchema) -> Result<NodeId, E>;

    /// Get info regarding a visible key value store
    fn key_value_store_get_info(&mut self, node_id: &NodeId) -> Result<KeyValueStoreSchema, E>;

    /// Lock a key value store entry for reading/writing
    fn key_value_store_open_entry(
        &mut self,
        node_id: &NodeId,
        key: &Vec<u8>,
        flags: LockFlags,
    ) -> Result<KeyValueEntryHandle, E>;

    fn key_value_store_remove_entry(
        &mut self,
        node_id: &NodeId,
        key: &Vec<u8>,
    ) -> Result<Vec<u8>, E>;
}
