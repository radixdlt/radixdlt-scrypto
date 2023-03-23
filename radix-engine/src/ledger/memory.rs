use crate::kernel::interpreters::ScryptoInterpreter;
use crate::ledger::WriteableSubstateStore;
use crate::ledger::*;
use crate::types::*;
use crate::wasm::WasmEngine;
use sbor::rust::ops::Bound::Included;

#[derive(Debug, PartialEq, Eq)]
pub struct TypedInMemorySubstateStore {
    substates: BTreeMap<Vec<u8>, IndexedScryptoValue>,
}

impl TypedInMemorySubstateStore {
    pub fn new() -> Self {
        Self {
            substates: BTreeMap::new(),
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
    fn get_substate(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Option<IndexedScryptoValue> {
        let substate_id = encode_substate_id(node_id, module_id, substate_key);
        self.substates.get(&substate_id).cloned()
    }
}

impl WriteableSubstateStore for TypedInMemorySubstateStore {
    fn put_substate(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
        substate_value: IndexedScryptoValue,
    ) {
        let substate_id = encode_substate_id(node_id, module_id, substate_key);
        self.substates.insert(substate_id, substate_value);
    }
}

impl QueryableSubstateStore for TypedInMemorySubstateStore {
    fn list_substates(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> BTreeMap<SubstateKey, IndexedScryptoValue> {
        let min = encode_substate_id(
            node_id,
            module_id,
            &SubstateKey::State(StateIdentifier::MIN),
        );
        let max = encode_substate_id(
            node_id,
            module_id,
            &SubstateKey::State(StateIdentifier::MAX),
        );
        self.substates
            .range::<Vec<u8>, _>((Included(&min), Included(&max)))
            .into_iter()
            .map(|(k, v)| (decode_substate_id(k).2, v.clone()))
            .collect()
    }
}
