use radix_engine_common::data::scrypto::{
    scrypto_decode, scrypto_encode, ScryptoDecode, ScryptoEncode,
};
use radix_engine_derive::{ManifestSbor, ScryptoSbor};
use sbor::rust::prelude::*;
use sbor::rust::vec::Vec;

#[derive(Clone, Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct SortedKey(pub u16, pub Vec<u8>);

impl SortedKey {
    pub fn new(sorted: u16, key: Vec<u8>) -> Self {
        Self(sorted, key)
    }
}

impl Into<Vec<u8>> for SortedKey {
    fn into(self) -> Vec<u8> {
        let mut bytes = self.0.to_be_bytes().to_vec();
        bytes.extend(self.1);
        bytes
    }
}

// TODO: Add locked entry interface
pub trait ClientActorSortedIndexApi<E> {
    /// Inserts an entry into a sorted index
    fn actor_outer_object_sorted_index_insert(
        &mut self,
        handle: u8,
        sorted_key: SortedKey,
        buffer: Vec<u8>,
    ) -> Result<(), E>;

    /// Inserts an entry into a sorted index
    fn actor_outer_object_sorted_index_insert_typed<V: ScryptoEncode>(
        &mut self,
        handle: u8,
        sorted_key: SortedKey,
        value: V,
    ) -> Result<(), E> {
        self.actor_outer_object_sorted_index_insert(
            handle,
            sorted_key,
            scrypto_encode(&value).unwrap(),
        )
    }

    /// Removes an entry from a sorted index
    fn actor_outer_object_sorted_index_remove(
        &mut self,
        handle: u8,
        sorted_key: &SortedKey,
    ) -> Result<Option<Vec<u8>>, E>;

    /// Removes an entry from a sorted index
    fn actor_outer_object_sorted_index_remove_typed<V: ScryptoDecode>(
        &mut self,
        handle: u8,
        sorted_key: &SortedKey,
    ) -> Result<Option<V>, E> {
        let rtn = self
            .actor_outer_object_sorted_index_remove(handle, sorted_key)?
            .map(|e| scrypto_decode(&e).unwrap());
        Ok(rtn)
    }

    /// Scans the first elements of count from a sorted index
    fn actor_sorted_index_scan(&mut self, handle: u8, count: u32) -> Result<Vec<Vec<u8>>, E>;

    /// Scans the first elements of count from a sorted index
    fn actor_sorted_index_scan_typed<S: ScryptoDecode>(
        &mut self,
        handle: u8,
        count: u32,
    ) -> Result<Vec<S>, E> {
        let entries = self
            .actor_sorted_index_scan(handle, count)?
            .into_iter()
            .map(|buf| {
                let typed: S = scrypto_decode(&buf).unwrap();
                typed
            })
            .collect();

        Ok(entries)
    }
}
