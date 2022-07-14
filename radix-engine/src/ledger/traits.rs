use sbor::rust::collections::*;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::*;
use scrypto::crypto::*;
use scrypto::engine::types::*;

pub trait QueryableSubstateStore {
    fn get_kv_store_entries(
        &self,
        component_address: ComponentAddress,
        kv_store_id: &KeyValueStoreId,
    ) -> HashMap<Vec<u8>, Vec<u8>>;
}

#[derive(Debug, Clone, Hash, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct PhysicalSubstateId(pub Hash, pub u32);

#[derive(Clone, Debug, Encode, Decode, TypeId)]
pub struct Substate {
    pub value: Vec<u8>,
    pub phys_id: PhysicalSubstateId,
}

#[derive(Debug)]
pub struct SubstateIdGenerator {
    tx_hash: Hash,
    count: u32,
}

impl SubstateIdGenerator {
    pub fn new(tx_hash: Hash) -> Self {
        Self { tx_hash, count: 0 }
    }

    pub fn next(&mut self) -> PhysicalSubstateId {
        let value = self.count;
        self.count = self.count + 1;
        PhysicalSubstateId(self.tx_hash.clone(), value)
    }
}

pub trait ReadableSubstateStore {
    fn get_substate(&self, address: &[u8]) -> Option<Substate>;
    fn get_space(&mut self, address: &[u8]) -> Option<PhysicalSubstateId>;

    // Temporary Encoded/Decoded interface
    fn get_decoded_substate<A: Encode, T: Decode>(
        &self,
        address: &A,
    ) -> Option<(T, PhysicalSubstateId)> {
        self.get_substate(&scrypto_encode(address))
            .map(|s| (scrypto_decode(&s.value).unwrap(), s.phys_id))
    }

    fn get_epoch(&self) -> u64;
}

pub trait WriteableSubstateStore {
    fn put_substate(&mut self, address: &[u8], substate: Substate);
    fn put_space(&mut self, address: &[u8], phys_id: PhysicalSubstateId);
    fn set_epoch(&mut self, epoch: u64);
}
