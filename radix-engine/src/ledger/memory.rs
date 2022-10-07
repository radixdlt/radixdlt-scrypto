use crate::ledger::*;
use crate::ledger::{OutputValue, WriteableSubstateStore};
use crate::model::Substate;
use crate::types::*;

/// A substate store that stores all typed substates in host memory.
#[derive(Debug, PartialEq, Eq)]
pub struct TypedInMemorySubstateStore {
    substates: HashMap<SubstateId, OutputValue>,
}

impl TypedInMemorySubstateStore {
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

impl Default for TypedInMemorySubstateStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadableSubstateStore for TypedInMemorySubstateStore {
    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue> {
        self.substates.get(substate_id).cloned()
    }
}

impl WriteableSubstateStore for TypedInMemorySubstateStore {
    fn put_substate(&mut self, substate_id: SubstateId, substate: OutputValue) {
        self.substates.insert(substate_id, substate);
    }
}

impl QueryableSubstateStore for TypedInMemorySubstateStore {
    fn get_kv_store_entries(
        &self,
        kv_store_id: &KeyValueStoreId,
    ) -> HashMap<sbor::rust::vec::Vec<u8>, Substate> {
        self.substates
            .iter()
            .filter_map(|(key, value)| {
                if let SubstateId(
                    RENodeId::KeyValueStore(id),
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key)),
                ) = key
                {
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
