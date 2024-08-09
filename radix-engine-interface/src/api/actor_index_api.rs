use crate::api::ActorStateHandle;
use radix_common::prelude::*;
use radix_engine_interface::api::CollectionIndex;

/// Api to manage an iterable index
pub trait SystemActorIndexApi<E> {
    /// Inserts an entry into an index
    fn actor_index_insert(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        key: ScryptoUnvalidatedRawValue,
        buffer: ScryptoUnvalidatedRawValue,
    ) -> Result<(), E>;

    /// Inserts an entry into an index
    fn actor_index_insert_typed<K: ScryptoEncode, V: ScryptoEncode>(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        key: K,
        value: V,
    ) -> Result<(), E> {
        self.actor_index_insert(
            object_handle,
            collection_index,
            scrypto_encode_to_value(&key).unwrap().into_unvalidated(),
            scrypto_encode_to_value(&value).unwrap().into_unvalidated(),
        )
    }

    /// Removes an entry from an index
    fn actor_index_remove(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        key: ScryptoUnvalidatedRawValue,
    ) -> Result<Option<ScryptoOwnedRawValue>, E>;

    /// Removes an entry from an index
    fn actor_index_remove_typed<V: ScryptoDecode>(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        key: ScryptoUnvalidatedRawValue,
    ) -> Result<Option<V>, E> {
        let rtn = self
            .actor_index_remove(object_handle, collection_index, key)?
            .map(|e| e.decode_as().unwrap());
        Ok(rtn)
    }

    /// Scans arbitrary elements of count from an index
    fn actor_index_scan_keys(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        limit: u32,
    ) -> Result<Vec<ScryptoOwnedRawValue>, E>;

    /// Scans arbitrary elements of count from an index
    fn actor_index_scan_keys_typed<K: ScryptoDecode>(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        limit: u32,
    ) -> Result<Vec<K>, E> {
        let entries = self
            .actor_index_scan_keys(object_handle, collection_index, limit)?
            .into_iter()
            .map(|key| key.decode_as().unwrap())
            .collect();

        Ok(entries)
    }

    /// Removes and returns arbitrary elements of count from an index
    fn actor_index_drain(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        limit: u32,
    ) -> Result<Vec<(ScryptoOwnedRawValue, ScryptoOwnedRawValue)>, E>;

    /// Removes and returns arbitrary elements of count from an index
    fn actor_index_drain_typed<K: ScryptoDecode, V: ScryptoDecode>(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        limit: u32,
    ) -> Result<Vec<(K, V)>, E> {
        let entries = self
            .actor_index_drain(object_handle, collection_index, limit)?
            .into_iter()
            .map(|(key, value)| (key.decode_as().unwrap(), value.decode_as().unwrap()))
            .collect();

        Ok(entries)
    }
}
