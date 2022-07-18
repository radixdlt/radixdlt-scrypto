use sbor::rust::collections::HashMap;

use crate::engine::Address;
use crate::ledger::*;
use crate::ledger::{Substate, WriteableSubstateStore};

/// A substate store that stores all substates in host memory.
pub struct InMemorySubstateStore {
    substates: HashMap<Address, Substate>,
    spaces: HashMap<Address, PhysicalSubstateId>,
}

impl InMemorySubstateStore {
    pub fn new() -> Self {
        Self {
            substates: HashMap::new(),
            spaces: HashMap::new(),
        }
    }

    pub fn with_bootstrap() -> Self {
        let mut substate_store = Self::new();
        bootstrap(substate_store, scrypto::core::Network::LocalSimulator)
    }
}

impl Default for InMemorySubstateStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadableSubstateStore for InMemorySubstateStore {
    fn get_substate(&self, address: &Address) -> Option<Substate> {
        self.substates.get(address).cloned()
    }

    fn get_space(&self, address: &Address) -> Option<PhysicalSubstateId> {
        self.spaces.get(address).cloned()
    }
}

impl WriteableSubstateStore for InMemorySubstateStore {
    fn put_substate(&mut self, address: Address, substate: Substate) {
        self.substates.insert(address, substate);
    }

    fn put_space(&mut self, address: Address, phys_id: PhysicalSubstateId) {
        self.spaces.insert(address, phys_id);
    }
}
