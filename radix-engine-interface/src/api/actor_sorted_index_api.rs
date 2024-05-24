use crate::api::ActorStateHandle;
use radix_common::data::scrypto::{scrypto_decode, scrypto_encode, ScryptoDecode, ScryptoEncode};
use radix_common::types::SortedKey;
use radix_engine_interface::api::CollectionIndex;
use sbor::rust::prelude::*;
use sbor::rust::vec::Vec;

pub trait SystemActorSortedIndexApi<E> {
    /// Inserts an entry into a sorted index
    fn actor_sorted_index_insert(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        sorted_key: SortedKey,
        buffer: Vec<u8>,
    ) -> Result<(), E>;

    /// Inserts an entry into a sorted index
    fn actor_sorted_index_insert_typed<V: ScryptoEncode>(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        sorted_key: SortedKey,
        value: V,
    ) -> Result<(), E> {
        self.actor_sorted_index_insert(
            object_handle,
            collection_index,
            sorted_key,
            scrypto_encode(&value).unwrap(),
        )
    }

    /// Removes an entry from a sorted index
    fn actor_sorted_index_remove(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        sorted_key: &SortedKey,
    ) -> Result<Option<Vec<u8>>, E>;

    /// Removes an entry from a sorted index
    fn actor_sorted_index_remove_typed<V: ScryptoDecode>(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        sorted_key: &SortedKey,
    ) -> Result<Option<V>, E> {
        let rtn = self
            .actor_sorted_index_remove(object_handle, collection_index, sorted_key)?
            .map(|e| scrypto_decode(&e).unwrap());
        Ok(rtn)
    }

    /// Scans the first elements of count from a sorted index
    fn actor_sorted_index_scan(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        count: u32,
    ) -> Result<Vec<(SortedKey, Vec<u8>)>, E>;

    /// Scans the first elements of count from a sorted index
    fn actor_sorted_index_scan_typed<K: ScryptoDecode, V: ScryptoDecode>(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        count: u32,
    ) -> Result<Vec<(K, V)>, E> {
        let entries = self
            .actor_sorted_index_scan(object_handle, collection_index, count)?
            .into_iter()
            .map(|(key, buf)| {
                let typed_key: K = scrypto_decode(&key.1).unwrap();
                let typed_value: V = scrypto_decode(&buf).unwrap();
                (typed_key, typed_value)
            })
            .collect();

        Ok(entries)
    }
}
