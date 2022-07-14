use sbor::rust::collections::HashMap;
use sbor::rust::vec::Vec;
use scrypto::buffer::{scrypto_decode, scrypto_encode};

use crate::ledger::*;
use crate::ledger::{Output, WriteableSubstateStore};


/// A substate store that stores all substates in host memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InMemorySubstateStore {
    substates: HashMap<Vec<u8>, Vec<u8>>,
}

impl InMemorySubstateStore {
    pub fn new() -> Self {
        Self {
            substates: HashMap::new(),
        }
    }

    pub fn with_bootstrap() -> Self {
        let mut substate_store = Self::new();
        bootstrap(&mut substate_store);
        substate_store
    }
}

impl Default for InMemorySubstateStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadableSubstateStore for InMemorySubstateStore {
    fn get_substate(&self, address: &[u8]) -> Option<Output> {
        self.substates
            .get(address)
            .map(|bytes| scrypto_decode(bytes).unwrap())
    }

    fn get_space(&self, address: &[u8]) -> OutputId {
        self.substates
            .get(address)
            .map(|bytes| scrypto_decode(bytes).unwrap())
            .expect("Expected space does not exist")
    }
}

impl WriteableSubstateStore for InMemorySubstateStore {
    fn put_substate(&mut self, address: &[u8], substate: Output) {
        self.substates
            .insert(address.to_vec(), scrypto_encode(&substate));
    }

    fn put_space(&mut self, address: &[u8], phys_id: OutputId) {
        self.substates
            .insert(address.to_vec(), scrypto_encode(&phys_id));
    }
}
