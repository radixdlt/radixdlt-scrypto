use radix_engine_common::data::scrypto::{
    scrypto_decode, scrypto_encode, ScryptoDecode, ScryptoEncode,
};
use radix_engine_common::types::*;
use sbor::rust::prelude::*;
use sbor::rust::vec::Vec;

/// Api to manage an iterable index
pub trait ClientIndexApi<E> {
    /// Creates a new index
    fn new_index(&mut self) -> Result<NodeId, E>;

    /// Inserts an entry into an index
    fn insert_into_index(
        &mut self,
        node_id: &NodeId,
        key: Vec<u8>,
        buffer: Vec<u8>,
    ) -> Result<(), E>;

    /// Inserts an entry into an index
    fn insert_typed_into_index<V: ScryptoEncode>(
        &mut self,
        node_id: &NodeId,
        key: Vec<u8>,
        value: V,
    ) -> Result<(), E> {
        self.insert_into_index(node_id, key, scrypto_encode(&value).unwrap())
    }

    /// Removes an entry from an index
    fn remove_from_index(&mut self, node_id: &NodeId, key: Vec<u8>) -> Result<Option<Vec<u8>>, E>;

    /// Removes an entry from an index
    fn remove_typed_from_index<V: ScryptoDecode>(
        &mut self,
        node_id: &NodeId,
        key: Vec<u8>,
    ) -> Result<Option<V>, E> {
        let rtn = self
            .remove_from_index(node_id, key)?
            .map(|e| scrypto_decode(&e).unwrap());
        Ok(rtn)
    }

    /// Scans arbitrary elements of count from an index
    fn scan_index(&mut self, node_id: &NodeId, count: u32) -> Result<Vec<Vec<u8>>, E>;

    /// Scans arbitrary elements of count from an index
    fn scan_typed_index<S: ScryptoDecode>(
        &mut self,
        node_id: &NodeId,
        count: u32,
    ) -> Result<Vec<S>, E> {
        let entries = self
            .scan_index(node_id, count)?
            .into_iter()
            .map(|buf| {
                let typed: S = scrypto_decode(&buf).unwrap();
                typed
            })
            .collect();

        Ok(entries)
    }

    /// Removes and returns arbitrary elements of count from an index
    fn take(&mut self, node_id: &NodeId, count: u32) -> Result<Vec<Vec<u8>>, E>;

    /// Removes and returns arbitrary elements of count from an index
    fn take_typed<S: ScryptoDecode>(&mut self, node_id: &NodeId, count: u32) -> Result<Vec<S>, E> {
        let entries = self
            .take(node_id, count)?
            .into_iter()
            .map(|buf| {
                let typed: S = scrypto_decode(&buf).unwrap();
                typed
            })
            .collect();

        Ok(entries)
    }
}
