use radix_engine_common::data::scrypto::{scrypto_decode, ScryptoDecode};
use radix_engine_interface::api::{LockFlags, ObjectHandle};
use radix_engine_interface::api::key_value_entry_api::KeyValueEntryHandle;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;

pub trait ClientActorKeyValueEntryApi<E: Debug> {
    fn actor_lock_key_value_entry(
        &mut self,
        object_handle: ObjectHandle,
        partition_index: u8,
        key: &[u8],
        flags: LockFlags,
    ) -> Result<KeyValueEntryHandle, E>;

    fn actor_remove_key_value_entry(
        &mut self,
        object_handle: ObjectHandle,
        partition_index: u8,
        key: &Vec<u8>,
    ) -> Result<Vec<u8>, E>;

    fn actor_remove_key_value_entry_typed<V: ScryptoDecode>(
        &mut self,
        object_handle: ObjectHandle,
        partition_index: u8,
        key: &Vec<u8>,
    ) -> Result<Option<V>, E> {
        let removed = self.actor_remove_key_value_entry(object_handle, partition_index, key)?;
        let rtn = scrypto_decode(&removed).unwrap();
        Ok(rtn)
    }
}