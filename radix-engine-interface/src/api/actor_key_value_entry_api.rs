use radix_common::data::scrypto::{scrypto_decode, ScryptoDecode};
use radix_engine_interface::api::key_value_entry_api::KeyValueEntryHandle;
use radix_engine_interface::api::{ActorStateHandle, CollectionIndex, LockFlags};
use sbor::rust::vec::Vec;

pub trait SystemActorKeyValueEntryApi<E> {
    /// Returns a handle for a specified key value entry in a collection. If an invalid collection
    /// index or key is passed an error is returned.
    fn actor_open_key_value_entry(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        key: &Vec<u8>,
        flags: LockFlags,
    ) -> Result<KeyValueEntryHandle, E>;

    /// Removes an entry from a collection. If an invalid collection index or key is passed an
    /// error is returned, otherwise the encoding of a value of the entry is returned.
    fn actor_remove_key_value_entry(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        key: &Vec<u8>,
    ) -> Result<Vec<u8>, E>;

    /// Removes an entry from a collection. If an invalid collection index or key is passed an
    /// error is returned, otherwise the value of the entry is returned.
    fn actor_remove_key_value_entry_typed<V: ScryptoDecode>(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        key: &Vec<u8>,
    ) -> Result<Option<V>, E> {
        let removed = self.actor_remove_key_value_entry(object_handle, collection_index, key)?;
        let rtn = scrypto_decode(&removed).unwrap();
        Ok(rtn)
    }
}
