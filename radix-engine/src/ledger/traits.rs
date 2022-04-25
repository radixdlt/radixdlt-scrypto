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

#[derive(Clone, Debug, Encode, Decode, TypeId)]
pub struct Substate {
    pub value: Vec<u8>,
    pub phys_id: (Hash, u32),
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

    pub fn next(&mut self) -> (Hash, u32) {
        let value = self.count;
        self.count = self.count + 1;
        (self.tx_hash.clone(), value)
    }
}


/// A ledger stores all transactions and substates.
pub trait ReadableSubstateStore {
    fn get_substate<T: Encode>(&self, address: &T) -> Option<Substate>;

    fn get_child_substate<T: Encode>(&self, address: &T, key: &[u8]) -> Option<Substate>;

    // Temporary Encoded/Decoded interface
    fn get_decoded_substate<A: Encode, T: Decode>(&self, address: &A) -> Option<(T, (Hash, u32))> {
        self.get_substate(address)
            .map(|s| (scrypto_decode(&s.value).unwrap(), s.phys_id))
    }

    fn get_decoded_child_substate<A: Encode, K: Encode, T: Decode>(
        &self,
        address: &A,
        key: &K,
    ) -> Option<(T, (Hash, u32))> {
        let child_key = &scrypto_encode(key);
        self.get_child_substate(address, child_key)
            .map(|s| (scrypto_decode(&s.value).unwrap(), s.phys_id))
    }

    fn get_decoded_grand_child_substate<A: Encode, C: Encode>(
        &self,
        address: &A,
        child_key: &C,
        grand_child_key: &[u8],
    ) -> Option<(Vec<u8>, (Hash, u32))> {
        let mut key = scrypto_encode(child_key);
        key.extend(grand_child_key.to_vec());
        self.get_child_substate(address, &key)
            .map(|s| (s.value, s.phys_id))
    }

    fn get_epoch(&self) -> u64;

    // TODO: redefine what nonce is and how it's updated
    // For now, we bump nonce only when a transaction has been committed
    // or when an account is created (for testing).
    fn get_nonce(&self) -> u64;
}

pub trait WriteableSubstateStore {
    fn put_substate<T: Encode>(&mut self, address: &T, substate: Substate);
    fn put_child_substate<T: Encode>(&mut self, address: &T, key: &[u8], substate: Substate);
    fn put_encoded_substate<A: Encode, V: Encode>(
        &mut self,
        address: &A,
        value: &V,
        phys_id: (Hash, u32),
    ) {
        self.put_substate(
            address,
            Substate {
                value: scrypto_encode(value),
                phys_id,
            },
        );
    }
    fn put_encoded_grand_child_substate<A: Encode, C: Encode>(
        &mut self,
        address: &A,
        child_key: &C,
        grand_child_key: &[u8],
        value: &[u8],
        phys_id: (Hash, u32),
    ) {
        let mut key = scrypto_encode(child_key);
        key.extend(grand_child_key.to_vec());
        self.put_child_substate(
            address,
            &key,
            Substate {
                value: value.to_vec(),
                phys_id,
            },
        );
    }

    fn put_encoded_child_substate<A: Encode, K: Encode, V: Encode>(
        &mut self,
        address: &A,
        key: &K,
        value: &V,
        phys_id: (Hash, u32),
    ) {
        let child_key = &scrypto_encode(key);
        self.put_child_substate(
            address,
            child_key,
            Substate {
                value: scrypto_encode(value),
                phys_id,
            },
        );
    }

    fn set_epoch(&mut self, epoch: u64);

    fn increase_nonce(&mut self);
}