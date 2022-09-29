use sbor::rust::collections::HashMap;
use scrypto::engine::types::KeyValueStoreId;

use crate::{engine::Track, fee::FeeReserve, model::KeyValueStoreEntrySubstate};

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

    pub fn get_loaded(&mut self, key: &[u8]) -> KeyValueStoreEntrySubstate {
        self.loaded_entries
            .get(key)
            .cloned()
            .unwrap_or(KeyValueStoreEntrySubstate(None)) // virtualization
    }

    pub fn get<'s, R: FeeReserve>(
        &self,
        key: &[u8],
        store_id: KeyValueStoreId,
        track: &mut Track<'s, R>,
    ) -> KeyValueStoreEntrySubstate {
        if !self.loaded_entries.contains_key(key) {
            let substate = track.read_key_value(
                scrypto::engine::types::SubstateId::KeyValueStoreSpace(store_id),
                key.to_vec(),
            );
            self.loaded_entries.insert(key.to_vec(), substate.into());
        }

        self.loaded_entries.get(key).unwrap().clone()
    }
}
