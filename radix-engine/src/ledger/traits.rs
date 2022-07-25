use crate::engine::Substate;
use sbor::rust::collections::*;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::crypto::*;
use scrypto::engine::types::*;

use crate::engine::Address;

pub trait QueryableSubstateStore {
    fn get_kv_store_entries(
        &self,
        component_address: ComponentAddress,
        kv_store_id: &KeyValueStoreId,
    ) -> HashMap<Vec<u8>, Substate>;
}

#[derive(Debug, Clone, Hash, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct OutputId(pub Hash, pub u32);

#[derive(Debug, Clone, Encode, Decode, TypeId)]
pub struct Output {
    pub substate: Substate,
    pub output_id: OutputId,
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

pub trait ReadableSubstateStore {
    fn get_substate(&self, address: &Address) -> Option<Output>;
    fn get_space(&self, address: &Address) -> OutputId;
}

pub trait WriteableSubstateStore {
    fn put_substate(&mut self, address: Address, substate: Output);
    fn put_space(&mut self, address: Address, output_id: OutputId);
}
