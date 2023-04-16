use radix_engine_common::data::scrypto::{
    scrypto_decode, scrypto_encode, ScryptoDecode, ScryptoEncode,
};
use crate::types::*;
use radix_engine_common::types::*;
use radix_engine_derive::{ManifestSbor, ScryptoSbor};
use sbor::rust::prelude::*;
use sbor::rust::vec::Vec;

pub trait ClientIterableStoreApi<E> {
    /// Creates a new iterable store
    fn new_iterable_store(&mut self) -> Result<NodeId, E>;

    /// Inserts an entry into a sorted store
    fn insert_into_iterable_store(
        &mut self,
        node_id: &NodeId,
        key: SubstateKey,
        buffer: Vec<u8>,
    ) -> Result<(), E>;

    /// Inserts an entry into a sorted store
    fn insert_typed_into_iterable_store<V: ScryptoEncode>(
        &mut self,
        node_id: &NodeId,
        key: SubstateKey,
        value: V,
    ) -> Result<(), E> {
        self.insert_into_iterable_store(node_id, key, scrypto_encode(&value).unwrap())
    }

    /// Removes an entry from a sorted store
    fn remove_from_iterable_store(
        &mut self,
        node_id: &NodeId,
        key: &SubstateKey,
    ) -> Result<Option<Vec<u8>>, E>;

    /// Removes an entry from a sorted store
    fn remove_typed_from_iterable_store<V: ScryptoDecode>(
        &mut self,
        node_id: &NodeId,
        key: &SubstateKey,
    ) -> Result<Option<V>, E> {
        let rtn = self
            .remove_from_iterable_store(node_id, key)?
            .map(|e| scrypto_decode(&e).unwrap());
        Ok(rtn)
    }

    /// Scans the first elements of count from an iterable store
    fn scan_iterable_store(&mut self, node_id: &NodeId, count: u32) -> Result<Vec<Vec<u8>>, E>;

    /// Scans the first elements of count from an iterable store
    fn scap_typed_iterable_store<S: ScryptoDecode>(
        &mut self,
        node_id: &NodeId,
        count: u32,
    ) -> Result<Vec<S>, E> {
        let entries = self
            .scan_iterable_store(node_id, count)?
            .into_iter()
            .map(|buf| {
                let typed: S = scrypto_decode(&buf).unwrap();
                typed
            })
            .collect();

        Ok(entries)
    }

    fn take(&mut self, node_id: &NodeId, count: u32) -> Result<Vec<Vec<u8>>, E>;

    fn take_typed<S: ScryptoDecode>(
        &mut self,
        node_id: &NodeId,
        count: u32,
    ) -> Result<Vec<S>, E> {
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
