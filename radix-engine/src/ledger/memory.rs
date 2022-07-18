use sbor::rust::collections::{HashMap, HashSet};
use sbor::rust::vec::Vec;

use crate::engine::Address;
use crate::ledger::{bootstrap, ReadableSubstateStore, WriteableSubstateStore};

/// A substate store that stores all substates in host memory.
pub struct InMemorySubstateStore {
    substates: HashMap<Address, Vec<u8>>,
    spaces: HashSet<Address>,
}

impl InMemorySubstateStore {
    pub fn new() -> Self {
        Self {
            substates: HashMap::new(),
            spaces: HashSet::new(),
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
    fn get_substate(&mut self, address: &Address) -> Option<Vec<u8>> {
        self.substates.get(address).cloned()
    }

    fn get_space(&mut self, address: &Address) -> bool {
        self.spaces.contains(address)
    }
}

impl WriteableSubstateStore for InMemorySubstateStore {
    fn put_substate(&mut self, address: Address, substate: Vec<u8>) {
        self.substates.insert(address, substate);
    }

    fn put_space(&mut self, address: Address) {
        self.spaces.insert(address);
    }
}
