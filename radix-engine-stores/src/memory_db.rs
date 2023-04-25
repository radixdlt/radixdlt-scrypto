use crate::interface::*;
use sbor::rust::ops::Bound::Included;
use sbor::rust::prelude::*;
use std::ops::Bound::Unbounded;

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
    fn get_substate(&self, index_id: &Vec<u8>, db_key: &Vec<u8>) -> Option<Vec<u8>> {
        let key = encode_substate_id(index_id, db_key);
        self.substates.get(&key).map(|value| value.clone())
    }

    fn list_substates(
        &self,
        index_id: &Vec<u8>,
    ) -> Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + '_> {
        let start = encode_substate_id(index_id, &vec![0]);
        let index_id = index_id.clone();
        let iter = self
            .substates
            .range((Included(start), Unbounded))
            .map(|(k, v)| {
                let id = decode_substate_id(k).expect("Failed to decode substate ID");
                (id.0, id.1, v)
            })
            .take_while(move |(i, ..)| index_id.eq(i))
            .into_iter()
            .map(|(_, db_key, value)| (db_key, value.clone()));

        Box::new(iter)
    }
}

impl CommittableSubstateDatabase for InMemorySubstateDatabase {
    fn commit(&mut self, database_updates: &DatabaseUpdates) {
        for (index_id, index_updates) in &database_updates.database_updates {
            for (db_key, update) in index_updates {
                let substate_id = encode_substate_id(index_id, db_key);
                match update {
                    DatabaseUpdate::Set(substate_value) => {
                        self.substates.insert(substate_id, substate_value.clone());
                    }
                    DatabaseUpdate::Delete => {
                        self.substates.remove(&substate_id);
                    }
                }
            }
        }
    }
}
