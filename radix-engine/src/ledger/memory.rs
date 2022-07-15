use sbor::rust::collections::HashMap;
use sbor::rust::vec::Vec;
use scrypto::buffer::{scrypto_decode, scrypto_encode};

use crate::engine::Address;
use crate::ledger::*;
use crate::ledger::{Substate, WriteableSubstateStore};

/// A substate store that stores all substates in host memory.
pub struct InMemorySubstateStore {
    substates: HashMap<Address, Vec<u8>>,
}

impl InMemorySubstateStore {
    pub fn new() -> Self {
        Self {
            substates: HashMap::new(),
        }
    }

    pub fn with_bootstrap() -> Self {
        let mut substate_store = Self::new();
        bootstrap(&mut substate_store, scrypto::core::Network::LocalSimulator);
        substate_store
    }
}

impl Default for InMemorySubstateStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadableSubstateStore for InMemorySubstateStore {
    fn get_substate(&self, address: &Address) -> Option<Substate> {
        self.substates
            .get(address)
            .map(|bytes| scrypto_decode(bytes).unwrap())
    }

    fn get_space(&mut self, address: &Address) -> Option<PhysicalSubstateId> {
        self.substates
            .get(address)
            .map(|bytes| scrypto_decode(bytes).unwrap())
    }
}

impl WriteableSubstateStore for InMemorySubstateStore {
    fn put_substate(&mut self, address: Address, substate: Substate) {
        self.substates.insert(address, scrypto_encode(&substate));
    }

    fn put_space(&mut self, address: Address, phys_id: PhysicalSubstateId) {
        self.substates.insert(address, scrypto_encode(&phys_id));
    }
}
