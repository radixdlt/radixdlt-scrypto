use radix_engine_common::data::scrypto::{
    scrypto_decode, scrypto_encode, ScryptoDecode, ScryptoEncode,
};
use radix_engine_common::types::*;
use sbor::rust::prelude::*;
use sbor::rust::vec::Vec;

pub trait ClientIterableStoreApi<E> {
    /// Creates a new iterable store
    fn new_iterable_store(&mut self) -> Result<NodeId, E>;

    /// Inserts an entry into an iterable store
    fn insert_into_iterable_store(
        &mut self,
        node_id: &NodeId,
        key: SubstateKey,
        buffer: Vec<u8>,
    ) -> Result<(), E>;

    /// Inserts an entry into an iterable store
    fn insert_typed_into_iterable_store<V: ScryptoEncode>(
        &mut self,
        node_id: &NodeId,
        key: SubstateKey,
        value: V,
    ) -> Result<(), E> {
        self.insert_into_iterable_store(node_id, key, scrypto_encode(&value).unwrap())
    }

    /// Removes an entry from an iterable store
    fn remove_from_iterable_store(
        &mut self,
        node_id: &NodeId,
        key: &SubstateKey,
    ) -> Result<Option<Vec<u8>>, E>;

    /// Removes an entry from an iterable store
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

    /// Scans arbitrary elements of count from an iterable store
    fn scan_iterable_store(&mut self, node_id: &NodeId, count: u32) -> Result<Vec<(SubstateKey, Vec<u8>)>, E>;

    /// Scans arbitrary elements of count from an iterable store
    fn scap_typed_iterable_store<S: ScryptoDecode>(
        &mut self,
        node_id: &NodeId,
        count: u32,
    ) -> Result<Vec<(SubstateKey, S)>, E> {
        let entries = self
            .scan_iterable_store(node_id, count)?
            .into_iter()
            .map(|(key, buf)| {
                let typed: S = scrypto_decode(&buf).unwrap();
                (key, typed)
            })
            .collect();

        Ok(entries)
    }

    /// Removes and returns arbitrary elements of count from an iterable store
    fn take(&mut self, node_id: &NodeId, count: u32) -> Result<Vec<(SubstateKey, Vec<u8>)>, E>;

    /// Removes and returns arbitrary elements of count from an iterable store
    fn take_typed<S: ScryptoDecode>(
        &mut self,
        node_id: &NodeId,
        count: u32,
    ) -> Result<Vec<(SubstateKey, S)>, E> {
        let entries = self
            .take(node_id, count)?
            .into_iter()
            .map(|(key, buf)| {
                let typed: S = scrypto_decode(&buf).unwrap();
                (key, typed)
            })
            .collect();

        Ok(entries)
    }
}
