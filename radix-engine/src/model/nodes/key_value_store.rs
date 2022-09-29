 
use sbor::rust::collections::HashMap;

use crate::model::KeyValueStoreEntrySubstate;

#[derive(Debug)]
pub struct KeyValueStore {
    loaded_entries: HashMap<Vec<u8>, KeyValueStoreEntrySubstate>
}

impl KeyValueStore {
    pub fn new() -> Self {
        Self {
            loaded_entries: HashMap::new()
        }
    }
}
