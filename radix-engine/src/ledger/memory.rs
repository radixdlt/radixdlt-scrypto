use sbor::rust::collections::HashMap;

use crate::engine::SubstateId;
use crate::ledger::*;
use crate::ledger::{OutputValue, WriteableSubstateStore};

/// A substate store that stores all substates in host memory.
#[derive(Debug, PartialEq, Eq)]
pub struct InMemorySubstateStore {
    substates: HashMap<SubstateId, OutputValue>,
}

impl InMemorySubstateStore {
    pub fn new() -> Self {
        Self {
            substates: HashMap::new(),
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
    fn get_substate(&self, address: &SubstateId) -> Option<OutputValue> {
        self.substates.get(address).cloned()
    }
}

impl WriteableSubstateStore for InMemorySubstateStore {
    fn put_substate(&mut self, address: SubstateId, substate: OutputValue) {
        self.substates.insert(address, substate);
    }
}
