use radix_engine_common::data::scrypto::{
    scrypto_decode, scrypto_encode, ScryptoDecode, ScryptoEncode,
};
use radix_engine_common::types::*;
use radix_engine_interface::api::LockFlags;
use sbor::rust::prelude::*;
use scrypto_schema::KeyValueStoreSchema;

pub type KeyValueEntryLockHandle = u32;

// TODO: Add locked entry interface rather than using substate api
pub trait ClientKeyValueStoreApi<E> {
    /// Creates a new key value store with a given schema
    fn key_value_store_new(&mut self, schema: KeyValueStoreSchema) -> Result<NodeId, E>;

    /// Get info regarding a visible key value store
    fn key_value_store_get_info(&mut self, node_id: &NodeId) -> Result<KeyValueStoreSchema, E>;

    /// Lock a key value store entry for reading/writing
    fn key_value_store_lock_entry(
        &mut self,
        node_id: &NodeId,
        key: &Vec<u8>,
        flags: LockFlags,
    ) -> Result<KeyValueEntryLockHandle, E>;

    // TODO: Add specific kv store read lock apis

    // TODO: Change return to Option<Vec<u8>>
    fn key_value_entry_get(&mut self, handle: KeyValueEntryLockHandle) -> Result<Vec<u8>, E>;

    fn key_value_entry_get_typed<S: ScryptoDecode>(
        &mut self,
        handle: KeyValueEntryLockHandle,
    ) -> Result<S, E> {
        let buffer = self.key_value_entry_get(handle)?;
        let value: S = scrypto_decode(&buffer).unwrap();
        Ok(value)
    }

    fn key_value_entry_set(
        &mut self,
        handle: KeyValueEntryLockHandle,
        buffer: Vec<u8>,
    ) -> Result<(), E>;

    fn key_value_entry_set_typed<S: ScryptoEncode>(
        &mut self,
        handle: KeyValueEntryLockHandle,
        value: S,
    ) -> Result<(), E> {
        let buffer = scrypto_encode(&value).unwrap();
        self.key_value_entry_set(handle, buffer)
    }

    fn key_value_entry_lock_release(&mut self, handle: KeyValueEntryLockHandle) -> Result<(), E>;
}
