use std::ops::Bound::Unbounded;
use itertools::Itertools;
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
        db_key: &Vec<u8>,
    ) -> Option<Vec<u8>> {
        let key = encode_substate_id(node_id, module_id, db_key);
        self.substates
            .get(&key)
            .map(|value| value.clone())
    }

    fn list_substates(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + '_> {
        let start = encode_substate_id(node_id, module_id, &SubstateKey::min().into());
        let node_id = node_id.clone();
        let iter = self
            .substates
            .range((Included(start), Unbounded))
            .map(|(k, v)| {
                let id = decode_substate_id(k).expect("Failed to decode substate ID");
                (id.0, id.1, id.2, v)
            })
            .take_while(move |(n, m, ..)| node_id.eq(n) && m.eq(&module_id))
            .into_iter()
            .map(|(_, _, db_key, value)| {
                (db_key, value.clone())
            });

        Box::new(iter)
    }
}

impl CommittableSubstateDatabase for InMemorySubstateDatabase {
    fn commit(&mut self, state_changes: &StateUpdates) {
        for ((node_id, module_id, db_key), substate_change) in &state_changes.substate_changes
        {
            let substate_id = encode_substate_id(node_id, *module_id, db_key);
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
