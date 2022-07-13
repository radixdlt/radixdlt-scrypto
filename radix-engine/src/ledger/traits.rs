use sbor::rust::collections::*;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::*;
use scrypto::crypto::*;
use scrypto::engine::types::*;
use crate::engine::Substate;

pub trait QueryableSubstateStore {
    fn get_kv_store_entries(
        &self,
        component_address: ComponentAddress,
        kv_store_id: &KeyValueStoreId,
    ) -> HashMap<Vec<u8>, Substate>;
}

#[derive(Debug, Clone, Hash, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct OutputId(pub Hash, pub u32);

#[derive(Debug, Encode, Decode, TypeId)]
pub struct Output {
    pub value: Substate,
    pub phys_id: OutputId,
}

#[derive(Debug)]
pub struct OutputIdGenerator {
    tx_hash: Hash,
    count: u32,
}

impl OutputIdGenerator {
    pub fn new(tx_hash: Hash) -> Self {
        Self { tx_hash, count: 0 }
    }

    pub fn next(&mut self) -> OutputId {
        let value = self.count;
        self.count = self.count + 1;
        OutputId(self.tx_hash.clone(), value)
    }
}

/// A ledger stores all transactions and substates.
pub trait ReadableSubstateStore {
    fn get_substate(&self, address: &[u8]) -> Option<Output>;
    fn get_space(&mut self, address: &[u8]) -> Option<OutputId>;

    // Temporary Encoded/Decoded interface
    fn get_decoded_substate<A: Encode, T: From<Substate>>(
        &self,
        address: &A,
    ) -> Option<T> {
        self.get_substate(&scrypto_encode(address))
            .map(|s| s.value.into())
    }
}

pub trait WriteableSubstateStore {
    fn put_substate(&mut self, address: &[u8], substate: Output);
    fn put_space(&mut self, address: &[u8], phys_id: OutputId);
}
