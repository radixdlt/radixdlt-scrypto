use crate::interface::*;
use radix_engine_interface::types::*;
use sbor::rust::ops::Bound::Included;
use sbor::rust::prelude::*;

#[derive(Debug, PartialEq, Eq)]
pub struct InMemorySubstateDatabase {
    substates: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl InMemorySubstateDatabase {
    pub fn standard() -> Self {
        Self {
            substates: btreemap!(),
        }
    }
}

impl SubstateDatabase for InMemorySubstateDatabase {
    fn get_substate(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Option<Vec<u8>> {
        let key = encode_substate_id(node_id, module_id, substate_key);
        self.substates
            .get(&key)
            .map(|value| value.clone())
    }

    fn list_substates(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> Box<dyn Iterator<Item = (SubstateKey, Vec<u8>)> + '_> {
        let start = encode_substate_id(node_id, module_id, &SubstateKey::min());
        let end = encode_substate_id(node_id, module_id, &SubstateKey::max());
        let iter = self
            .substates
            .range((Included(start), Included(end)))
            .into_iter()
            .map(|(k, value)| {
                let (_, _, substate_key) =
                    decode_substate_id(k).expect("Failed to decode substate ID");
                (substate_key, value.clone())
            });

        Box::new(iter)
    }
}

impl CommittableSubstateDatabase for InMemorySubstateDatabase {
    fn commit(&mut self, state_changes: &StateUpdates) {
        for ((node_id, module_id, substate_key), substate_change) in &state_changes.substate_changes
        {
            let substate_id = encode_substate_id(node_id, *module_id, substate_key);
            match substate_change {
                StateUpdate::Set(substate_value) => {
                    self.substates
                        .insert(substate_id, substate_value.clone());
                }
                StateUpdate::Delete => {
                    self.substates.remove(&substate_id);
                }
            }
        }
    }
}
