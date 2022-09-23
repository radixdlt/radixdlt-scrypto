use crate::types::*;

use super::TryIntoSubstates;

#[derive(Debug)]
pub struct HeapKeyValueStore {
    pub store: HashMap<Vec<u8>, ScryptoValue>,
}

impl HeapKeyValueStore {
    pub fn new() -> Self {
        HeapKeyValueStore {
            store: HashMap::new(),
        }
    }

    pub fn put(&mut self, key: Vec<u8>, value: ScryptoValue) {
        self.store.insert(key, value);
    }

    pub fn get(&self, key: &[u8]) -> Option<ScryptoValue> {
        self.store.get(key).cloned()
    }
}

impl TryIntoSubstates for HeapKeyValueStore {
    type Error = ();

    fn try_into_substates(self) -> Result<Vec<crate::model::Substate>, Self::Error> {
        Ok(vec![])
    }
}
