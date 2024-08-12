use crate::api::ActorStateHandle;
use radix_common::prelude::*;
use radix_engine_interface::api::CollectionIndex;

pub trait SystemActorSortedIndexApi<E> {
    /// Inserts an entry into a sorted index
    fn actor_sorted_index_insert(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        sorted_key: UnvalidatedSortedKey,
        value: ScryptoUnvalidatedRawValue,
    ) -> Result<(), E>;

    /// Inserts an entry into a sorted index
    fn actor_sorted_index_insert_typed<V: ScryptoEncode>(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        sorted_key: UnvalidatedSortedKey,
        value: V,
    ) -> Result<(), E> {
        self.actor_sorted_index_insert(
            object_handle,
            collection_index,
            sorted_key,
            scrypto_encode_to_value(&value).unwrap().into_unvalidated(),
        )
    }

    /// Removes an entry from a sorted index
    fn actor_sorted_index_remove(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        sorted_key: UnvalidatedSortedKey,
    ) -> Result<Option<ScryptoOwnedRawValue>, E>;

    /// Removes an entry from a sorted index
    fn actor_sorted_index_remove_typed<V: ScryptoDecode>(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        sorted_key: UnvalidatedSortedKey,
    ) -> Result<Option<V>, E> {
        let rtn = self
            .actor_sorted_index_remove(object_handle, collection_index, sorted_key)?
            .map(|e| e.decode_as().unwrap());
        Ok(rtn)
    }

    /// Scans the first elements of count from a sorted index
    fn actor_sorted_index_scan(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        count: u32,
    ) -> Result<Vec<(SortedKey, ScryptoOwnedRawValue)>, E>;

    /// Scans the first elements of count from a sorted index
    fn actor_sorted_index_scan_typed<K: ScryptoDecode, V: ScryptoDecode>(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        count: u32,
    ) -> Result<Vec<(([u8; 2], K), V)>, E> {
        let entries = self
            .actor_sorted_index_scan(object_handle, collection_index, count)?
            .into_iter()
            .map(|(key, value)| {
                (
                    (key.0, key.1.decode_as().unwrap()),
                    value.decode_as().unwrap(),
                )
            })
            .collect();

        Ok(entries)
    }
}
