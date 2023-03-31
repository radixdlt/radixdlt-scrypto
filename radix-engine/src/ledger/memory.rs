use crate::kernel::interpreters::ScryptoInterpreter;
use crate::ledger::*;
use crate::ledger::{OutputValue, WriteableSubstateStore};
use crate::system::node_substates::PersistedSubstate;
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::types::{
    KeyValueStoreId, KeyValueStoreOffset, RENodeId, SubstateId, SubstateOffset,
};

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

    pub fn with_bootstrap<W: WasmEngine>(scrypto_interpreter: &ScryptoInterpreter<W>) -> Self {
        let mut substate_store = Self::new();
        bootstrap(&mut substate_store, scrypto_interpreter);
        substate_store
    }

    pub fn assert_eq(&self, other: &TypedInMemorySubstateStore) {
        for (id, val) in &self.substates {
            let maybe_val = other.substates.get(id);
            match maybe_val {
                None => panic!("Right missing substate: {:?}", id),
                Some(right_val) => {
                    if !val.eq(right_val) {
                        panic!(
                            "Substates not equal.\nLeft: {:?}\nRight: {:?}",
                            val, right_val
                        );
                    }
                }
            }
        }

        for (id, _) in &other.substates {
            let maybe_val = self.substates.get(id);
            match maybe_val {
                None => panic!("Left missing substate: {:?}", id),
                Some(..) => {}
            }
        }
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
    ) -> HashMap<Vec<u8>, PersistedSubstate> {
        self.substates
            .iter()
            .filter_map(|(substate_id, substate_value)| {
                if let SubstateId(
                    RENodeId::KeyValueStore(id),
                    NodeModuleId::SELF,
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(entry_id)),
                ) = substate_id
                {
                    if id == kv_store_id {
                        Some((entry_id.clone(), substate_value.substate.clone()))
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
