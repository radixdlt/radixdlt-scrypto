use sbor::rust::collections::HashMap;
use radix_engine::ledger::{bootstrap, OutputValue, ReadableSubstateStore, WriteableSubstateStore};
use scrypto::engine::types::{SubstateId};

use scrypto::buffer::*;
use scrypto::engine::types::*;

/// A substate store that stores all typed substates in host memory.
#[derive(Debug, PartialEq, Eq)]
pub struct SerializedInMemorySubstateStore {
    substates: HashMap<Vec<u8>, Vec<u8>>,
}

impl SerializedInMemorySubstateStore {
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

impl Default for SerializedInMemorySubstateStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadableSubstateStore for SerializedInMemorySubstateStore {
    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue> {
        self.substates.get(&scrypto_encode(substate_id)).map(|b| scrypto_decode(&b).unwrap())
    }
}

impl WriteableSubstateStore for SerializedInMemorySubstateStore {
    fn put_substate(&mut self, substate_id: SubstateId, substate: OutputValue) {
        self.substates.insert(scrypto_encode(&substate_id), scrypto_encode(&substate));
    }
}