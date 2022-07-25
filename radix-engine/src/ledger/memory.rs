use sbor::rust::collections::HashMap;

use crate::engine::Address;
use crate::ledger::*;
use crate::ledger::{Output, WriteableSubstateStore};

/// A substate store that stores all substates in host memory.
#[derive(Debug)]
pub struct InMemorySubstateStore {
    substates: HashMap<Address, Output>,
    spaces: HashMap<Address, OutputId>,
}

impl InMemorySubstateStore {
    pub fn new() -> Self {
        Self {
            substates: HashMap::new(),
            spaces: HashMap::new(),
        }
    }

    pub fn with_bootstrap() -> Self {
        let substate_store = Self::new();
        bootstrap(substate_store)
    }
}

impl Default for InMemorySubstateStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadableSubstateStore for InMemorySubstateStore {
    fn get_substate(&self, address: &Address) -> Option<Output> {
        self.substates.get(address).cloned()
    }

    fn get_space(&self, address: &Address) -> OutputId {
        self.spaces
            .get(address)
            .cloned()
            .expect("Expected space does not exist")
    }
}

impl WriteableSubstateStore for InMemorySubstateStore {
    fn put_substate(&mut self, address: Address, substate: Output) {
        self.substates.insert(address, substate);
    }

    fn put_space(&mut self, address: Address, output_id: OutputId) {
        self.spaces.insert(address, output_id);
    }
}
