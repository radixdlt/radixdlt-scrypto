use sbor::*;
use scrypto::buffer::*;
use scrypto::crypto::*;
use scrypto::engine::types::*;
use scrypto::rust::collections::*;
use scrypto::rust::vec::Vec;

pub trait QueryableSubstateStore {
    fn get_lazy_map_entries(
        &self,
        component_address: ComponentAddress,
        lazy_map_id: &LazyMapId,
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

/// A ledger stores all transactions and substates.
pub trait ReadableSubstateStore {
    fn get_substate(&self, address: &[u8]) -> Option<Substate>;
    fn get_child_substate(&self, address: &[u8], key: &[u8]) -> Option<Substate>;
    fn get_space(&mut self, address: &[u8]) -> Option<PhysicalSubstateId>;

    // Temporary Encoded/Decoded interface
    fn get_decoded_substate<A: Encode, T: Decode>(&self, address: &A) -> Option<(T, PhysicalSubstateId)> {
        self.get_substate(&scrypto_encode(address))
            .map(|s| (scrypto_decode(&s.value).unwrap(), s.phys_id))
    }

    fn get_decoded_child_substate<A: Encode, K: Encode, T: Decode>(
        &self,
        address: &A,
        key: &K,
    ) -> Option<(T, PhysicalSubstateId)> {
        let child_key = &scrypto_encode(key);
        self.get_child_substate(&scrypto_encode(address), child_key)
            .map(|s| (scrypto_decode(&s.value).unwrap(), s.phys_id))
    }

    fn get_decoded_grand_child_substate<A: Encode, C: Encode>(
        &self,
        address: &A,
        child_key: &C,
        grand_child_key: &[u8],
    ) -> Option<(Vec<u8>, PhysicalSubstateId)> {
        let mut key = scrypto_encode(child_key);
        key.extend(grand_child_key.to_vec());
        self.get_child_substate(&scrypto_encode(address), &key)
            .map(|s| (s.value, s.phys_id))
    }

    fn get_epoch(&self) -> u64;

    // TODO: redefine what nonce is and how it's updated
    // For now, we bump nonce only when a transaction has been committed
    // or when an account is created (for testing).
    fn get_nonce(&self) -> u64;
}

pub trait WriteableSubstateStore {
    fn put_substate(&mut self, address: &[u8], substate: Substate);
    fn put_child_substate(&mut self, address: &[u8], key: &[u8], substate: Substate);
    fn put_space(&mut self, address: &[u8], phys_id: PhysicalSubstateId);

    fn put_keyed_substate(&mut self, address: &[u8], value: Vec<u8>, phys_id: PhysicalSubstateId) {
        self.put_substate(address, Substate { value, phys_id });
    }

    fn put_encoded_substate<A: Encode, V: Encode>(
        &mut self,
        address: &A,
        value: &V,
        phys_id: PhysicalSubstateId,
    ) {
        self.put_substate(
            &scrypto_encode(address),
            Substate {
                value: scrypto_encode(value),
                phys_id,
            },
        );
    }

    fn put_encoded_child_substate<A: Encode, K: Encode, V: Encode>(
        &mut self,
        address: &A,
        key: &K,
        value: &V,
        phys_id: PhysicalSubstateId,
    ) {
        self.put_child_substate(
            &scrypto_encode(address),
            &scrypto_encode(key),
            Substate {
                value: scrypto_encode(value),
                phys_id,
            },
        );
    }

    fn set_epoch(&mut self, epoch: u64);

    fn increase_nonce(&mut self);
}