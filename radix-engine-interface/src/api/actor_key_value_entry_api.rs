use radix_common::prelude::*;
use radix_engine_interface::api::key_value_entry_api::KeyValueEntryHandle;
use radix_engine_interface::api::{ActorStateHandle, CollectionIndex, LockFlags};

pub trait SystemActorKeyValueEntryApi<E> {
    /// Returns a handle for a specified key value entry in a collection. If an invalid collection
    /// index or key is passed an error is returned.
    fn actor_open_key_value_entry(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        key: ScryptoUnvalidatedRawValue,
        flags: LockFlags,
    ) -> Result<KeyValueEntryHandle, E>;

    /// Returns a handle for a specified key value entry in a collection. If an invalid collection
    /// index or key is passed an error is returned.
    fn actor_open_key_value_entry_typed<T: ScryptoEncode>(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        key: &T,
        flags: LockFlags,
    ) -> Result<KeyValueEntryHandle, E> {
        let key = scrypto_encode_to_value(&key).unwrap().into_unvalidated();
        self.actor_open_key_value_entry(object_handle, collection_index, key, flags)
    }

    /// Removes an entry from a collection. If an invalid collection index or key is passed an
    /// error is returned, otherwise the encoding of a value of the entry is returned.
    fn actor_remove_key_value_entry(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        key: ScryptoUnvalidatedRawValue,
    ) -> Result<Option<ScryptoOwnedRawValue>, E>;

    /// Removes an entry from a collection. If an invalid collection index or key is passed an
    /// error is returned, otherwise the value of the entry is returned.
    fn actor_remove_key_value_entry_typed<T: ScryptoEncode, V: ScryptoDecode>(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        key: &T,
    ) -> Result<Option<V>, E> {
        let key = scrypto_encode_to_value(&key).unwrap().into_unvalidated();
        let removed = self.actor_remove_key_value_entry(object_handle, collection_index, key)?;
        let rtn = removed.map(|value| value.decode_as().unwrap());
        Ok(rtn)
    }

    /// Removes an entry from a collection. If an invalid collection index or key is passed an
    /// error is returned, otherwise an option is returned marking if the entry existed.
    fn actor_remove_key_value_entry_typed_ignore_return<T: ScryptoEncode>(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        key: &T,
    ) -> Result<Option<()>, E> {
        let key = scrypto_encode_to_value(&key).unwrap().into_unvalidated();
        let removed = self.actor_remove_key_value_entry(object_handle, collection_index, key)?;
        Ok(removed.map(|_| ()))
    }
}
