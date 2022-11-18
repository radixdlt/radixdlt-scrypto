use sbor::rust::collections::HashMap;

use crate::model::KeyValueStoreEntrySubstate;

#[derive(Debug)]
pub struct KeyValueStore {
    pub loaded_entries: HashMap<Vec<u8>, KeyValueStoreEntrySubstate>,
}

impl KeyValueStore {
    pub fn new() -> Self {
        Self {
            loaded_entries: HashMap::new(),
        }
    }

    pub fn put(&mut self, key: Vec<u8>, value: KeyValueStoreEntrySubstate) {
        self.loaded_entries.insert(key, value);
    }

    pub fn get(&mut self, key: &[u8]) -> Option<&KeyValueStoreEntrySubstate> {
        self.loaded_entries.get(key)
    }
}
