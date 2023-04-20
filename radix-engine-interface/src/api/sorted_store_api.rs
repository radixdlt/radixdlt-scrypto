use radix_engine_common::data::scrypto::{
    scrypto_decode, scrypto_encode, ScryptoDecode, ScryptoEncode,
};
use radix_engine_common::types::*;
use radix_engine_derive::{ManifestSbor, ScryptoSbor};
use sbor::rust::prelude::*;
use sbor::rust::vec::Vec;

#[derive(Clone, Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct SortedKey(u16, Vec<u8>);

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
pub trait ClientSortedStoreApi<E> {
    /// Creates a new sorted store
    fn new_sorted_store(&mut self) -> Result<NodeId, E>;

    /// Inserts an entry into a sorted store
    fn insert_into_sorted_store(
        &mut self,
        node_id: &NodeId,
        sorted_key: SortedKey,
        buffer: Vec<u8>,
    ) -> Result<(), E>;

    /// Inserts an entry into a sorted store
    fn insert_typed_into_sorted_store<V: ScryptoEncode>(
        &mut self,
        node_id: &NodeId,
        sorted_key: SortedKey,
        value: V,
    ) -> Result<(), E> {
        self.insert_into_sorted_store(node_id, sorted_key, scrypto_encode(&value).unwrap())
    }

    /// Removes an entry from a sorted store
    fn remove_from_sorted_store(
        &mut self,
        node_id: &NodeId,
        sorted_key: &SortedKey,
    ) -> Result<Option<Vec<u8>>, E>;

    /// Removes an entry from a sorted store
    fn remove_typed_from_sorted_store<V: ScryptoDecode>(
        &mut self,
        node_id: &NodeId,
        sorted_key: &SortedKey,
    ) -> Result<Option<V>, E> {
        let rtn = self
            .remove_from_sorted_store(node_id, sorted_key)?
            .map(|e| scrypto_decode(&e).unwrap());
        Ok(rtn)
    }

    /// Scans the first elements of count from a sorted store
    fn scan_sorted_store(&mut self, node_id: &NodeId, count: u32) -> Result<Vec<Vec<u8>>, E>;

    /// Scans the first elements of count from a sorted store
    fn scap_typed_sorted_store<S: ScryptoDecode>(
        &mut self,
        node_id: &NodeId,
        count: u32,
    ) -> Result<Vec<S>, E> {
        let entries = self
            .scan_sorted_store(node_id, count)?
            .into_iter()
            .map(|buf| {
                let typed: S = scrypto_decode(&buf).unwrap();
                typed
            })
            .collect();

        Ok(entries)
    }
}
