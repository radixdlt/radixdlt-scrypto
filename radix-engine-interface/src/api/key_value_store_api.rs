use crate::types::LockHandle;
use radix_engine_common::types::*;
use radix_engine_interface::api::LockFlags;
use sbor::rust::prelude::*;
use scrypto_schema::KeyValueStoreSchema;


pub type KeyValueEntryLockHandle = u32;

// TODO: Add locked entry interface rather than using substate api
pub trait ClientKeyValueStoreApi<E> {
    /// Creates a new key value store with a given schema
    fn new_key_value_store(&mut self, schema: KeyValueStoreSchema) -> Result<NodeId, E>;

    /// Get info regarding a visible key value store
    fn get_key_value_store_info(&mut self, node_id: &NodeId) -> Result<KeyValueStoreSchema, E>;

    /// Lock a key value store entry for reading/writing
    fn lock_key_value_store_entry(
        &mut self,
        node_id: &NodeId,
        key: &Vec<u8>,
        flags: LockFlags,
    ) -> Result<KeyValueEntryLockHandle, E>;


    // TODO: Change return to Option<Vec<u8>>
    fn key_value_entry_get(&mut self, handle: KeyValueEntryLockHandle) -> Result<Vec<u8>, E>;

    fn key_value_entry_set(
        &mut self,
        handle: KeyValueEntryLockHandle,
        buffer: Vec<u8>,
    ) -> Result<(), E>;

    // TODO: Add specific kv store read lock apis
}
