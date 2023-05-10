use crate::api::ObjectHandle;
use radix_engine_common::data::scrypto::{
    scrypto_decode, scrypto_encode, ScryptoDecode, ScryptoEncode,
};
use radix_engine_interface::api::CollectionIndex;
use sbor::rust::prelude::*;
use sbor::rust::vec::Vec;

/// Api to manage an iterable index
pub trait ClientActorIndexApi<E> {
    /// Inserts an entry into an index
    fn actor_index_insert(
        &mut self,
        object_handle: ObjectHandle,
        collection_index: CollectionIndex,
        key: Vec<u8>,
        buffer: Vec<u8>,
    ) -> Result<(), E>;

    /// Inserts an entry into an index
    fn actor_index_insert_typed<V: ScryptoEncode>(
        &mut self,
        object_handle: ObjectHandle,
        collection_index: CollectionIndex,
        key: Vec<u8>,
        value: V,
    ) -> Result<(), E> {
        self.actor_index_insert(
            object_handle,
            collection_index,
            key,
            scrypto_encode(&value).unwrap(),
        )
    }

    /// Removes an entry from an index
    fn actor_index_remove(
        &mut self,
        object_handle: ObjectHandle,
        collection_index: CollectionIndex,
        key: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, E>;

    /// Removes an entry from an index
    fn actor_index_remove_typed<V: ScryptoDecode>(
        &mut self,
        object_handle: ObjectHandle,
        collection_index: CollectionIndex,
        key: Vec<u8>,
    ) -> Result<Option<V>, E> {
        let rtn = self
            .actor_index_remove(object_handle, collection_index, key)?
            .map(|e| scrypto_decode(&e).unwrap());
        Ok(rtn)
    }

    /// Scans arbitrary elements of count from an index
    fn actor_index_scan(
        &mut self,
        object_handle: ObjectHandle,
        collection_index: CollectionIndex,
        count: u32,
    ) -> Result<Vec<Vec<u8>>, E>;

    /// Scans arbitrary elements of count from an index
    fn actor_index_scan_typed<S: ScryptoDecode>(
        &mut self,
        object_handle: ObjectHandle,
        collection_index: CollectionIndex,
        count: u32,
    ) -> Result<Vec<S>, E> {
        let entries = self
            .actor_index_scan(object_handle, collection_index, count)?
            .into_iter()
            .map(|buf| {
                let typed: S = scrypto_decode(&buf).unwrap();
                typed
            })
            .collect();

        Ok(entries)
    }

    /// Removes and returns arbitrary elements of count from an index
    fn actor_index_take(
        &mut self,
        object_handle: ObjectHandle,
        collection_index: CollectionIndex,
        count: u32,
    ) -> Result<Vec<Vec<u8>>, E>;

    /// Removes and returns arbitrary elements of count from an index
    fn actor_index_take_typed<S: ScryptoDecode>(
        &mut self,
        object_handle: ObjectHandle,
        collection_index: CollectionIndex,
        count: u32,
    ) -> Result<Vec<S>, E> {
        let entries = self
            .actor_index_take(object_handle, collection_index, count)?
            .into_iter()
            .map(|buf| {
                let typed: S = scrypto_decode(&buf).unwrap();
                typed
            })
            .collect();

        Ok(entries)
    }
}
