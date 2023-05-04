use radix_engine_common::data::scrypto::{
    scrypto_decode, scrypto_encode, ScryptoDecode, ScryptoEncode,
};
use sbor::rust::prelude::*;
use sbor::rust::vec::Vec;

/// Api to manage an iterable index
pub trait ClientActorIndexApi<E> {
    /// Inserts an entry into an index
    fn actor_index_insert(
        &mut self,
        index_handle: u8,
        key: Vec<u8>,
        buffer: Vec<u8>,
    ) -> Result<(), E>;

    /// Inserts an entry into an index
    fn actor_index_insert_typed<V: ScryptoEncode>(
        &mut self,
        index_handle: u8,
        key: Vec<u8>,
        value: V,
    ) -> Result<(), E> {
        self.actor_index_insert(index_handle, key, scrypto_encode(&value).unwrap())
    }

    /// Removes an entry from an index
    fn actor_index_remove(&mut self, index_handle: u8, key: Vec<u8>) -> Result<Option<Vec<u8>>, E>;

    /// Removes an entry from an index
    fn actor_index_remove_typed<V: ScryptoDecode>(
        &mut self,
        index_handle: u8,
        key: Vec<u8>,
    ) -> Result<Option<V>, E> {
        let rtn = self
            .actor_index_remove(index_handle, key)?
            .map(|e| scrypto_decode(&e).unwrap());
        Ok(rtn)
    }

    /// Scans arbitrary elements of count from an index
    fn actor_index_scan(&mut self, index_handle: u8, count: u32) -> Result<Vec<Vec<u8>>, E>;

    /// Scans arbitrary elements of count from an index
    fn actor_index_scan_typed<S: ScryptoDecode>(
        &mut self,
        index_handle: u8,
        count: u32,
    ) -> Result<Vec<S>, E> {
        let entries = self
            .actor_index_scan(index_handle, count)?
            .into_iter()
            .map(|buf| {
                let typed: S = scrypto_decode(&buf).unwrap();
                typed
            })
            .collect();

        Ok(entries)
    }

    /// Removes and returns arbitrary elements of count from an index
    fn actor_index_take(&mut self, index_handle: u8, count: u32) -> Result<Vec<Vec<u8>>, E>;

    /// Removes and returns arbitrary elements of count from an index
    fn actor_index_take_typed<S: ScryptoDecode>(
        &mut self,
        index_handle: u8,
        count: u32,
    ) -> Result<Vec<S>, E> {
        let entries = self
            .actor_index_take(index_handle, count)?
            .into_iter()
            .map(|buf| {
                let typed: S = scrypto_decode(&buf).unwrap();
                typed
            })
            .collect();

        Ok(entries)
    }
}
