use sbor::rust::collections::HashMap;
use scrypto::engine::types::{KeyValueStoreId, SubstateId};

use crate::engine::Substate;
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
    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue> {
        self.substates.get(substate_id).cloned()
    }
}

impl WriteableSubstateStore for InMemorySubstateStore {
    fn put_substate(&mut self, substate_id: SubstateId, substate: OutputValue) {
        self.substates.insert(substate_id, substate);
    }
}

impl QueryableSubstateStore for InMemorySubstateStore {
    fn get_kv_store_entries(&self, kv_store_id: &KeyValueStoreId) -> HashMap<sbor::rust::vec::Vec<u8>, Substate> {
        self.substates
            .iter()
            .filter_map(|(key, value)| {
                if let SubstateId::KeyValueStoreEntry(id, key) = key {
                    if id == kv_store_id {
                        Some((key.clone(), value.substate.clone()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }
}
