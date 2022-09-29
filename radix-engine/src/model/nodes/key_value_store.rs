use sbor::rust::collections::HashMap;
use scrypto::{engine::types::KeyValueStoreId, values::ScryptoValue};

use crate::{engine::Track, fee::FeeReserve, model::KeyValueStoreEntrySubstate};

#[derive(Debug)]
pub struct KeyValueStore {
    pub store_id: KeyValueStoreId,
    pub loaded_entries: HashMap<Vec<u8>, KeyValueStoreEntrySubstate>,
}

impl KeyValueStore {
    pub fn new(store_id: KeyValueStoreId) -> Self {
        Self {
            store_id,
            loaded_entries: HashMap::new(),
        }
    }

    pub fn put(&mut self, key: Vec<u8>, value: ScryptoValue) {
        self.loaded_entries
            .insert(key, KeyValueStoreEntrySubstate(Some(value.raw)));
    }

    pub fn get<'s, R: FeeReserve>(
        &self,
        key: &[u8],
        track: &mut Track<'s, R>,
    ) -> Option<ScryptoValue> {
        if !self.loaded_entries.contains_key(key) {
            let substate = track.read_key_value(
                scrypto::engine::types::SubstateId::KeyValueStoreSpace(self.store_id.clone()),
                key.to_vec(),
            );
            self.loaded_entries.insert(key.to_vec(), substate.into());
        }

        self.loaded_entries
            .get(key)
            .unwrap()
            .0
            .map(|raw| ScryptoValue::from_slice(&raw).unwrap())
    }
}
