use crate::api::ActorStateHandle;
use radix_common::data::scrypto::{scrypto_decode, scrypto_encode, ScryptoDecode, ScryptoEncode};
use radix_engine_interface::api::CollectionIndex;
use sbor::rust::prelude::*;
use sbor::rust::vec::Vec;

/// Api to manage an iterable index
pub trait SystemActorIndexApi<E> {
    /// Inserts an entry into an index
    fn actor_index_insert(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        key: Vec<u8>,
        buffer: Vec<u8>,
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
            scrypto_encode(&key).unwrap(),
            scrypto_encode(&value).unwrap(),
        )
    }

    /// Removes an entry from an index
    fn actor_index_remove(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        key: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, E>;

    /// Removes an entry from an index
    fn actor_index_remove_typed<V: ScryptoDecode>(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        key: Vec<u8>,
    ) -> Result<Option<V>, E> {
        let rtn = self
            .actor_index_remove(object_handle, collection_index, key)?
            .map(|e| scrypto_decode(&e).unwrap());
        Ok(rtn)
    }

    /// Scans arbitrary elements of count from an index
    fn actor_index_scan_keys(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        limit: u32,
    ) -> Result<Vec<Vec<u8>>, E>;

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
            .map(|key| {
                let key: K = scrypto_decode(&key).unwrap();
                key
            })
            .collect();

        Ok(entries)
    }

    /// Removes and returns arbitrary elements of count from an index
    fn actor_index_drain(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        limit: u32,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, E>;

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
            .map(|(key, value)| {
                let key: K = scrypto_decode(&key).unwrap();
                let value: V = scrypto_decode(&value).unwrap();
                (key, value)
            })
            .collect();

        Ok(entries)
    }
}
