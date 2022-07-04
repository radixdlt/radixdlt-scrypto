use sbor::rust::collections::*;
use sbor::rust::vec::Vec;
use scrypto::values::ScryptoValue;

#[derive(Debug)]
pub struct PreCommittedKeyValueStore {
    pub store: HashMap<Vec<u8>, ScryptoValue>,
}

impl PreCommittedKeyValueStore {
    pub fn new() -> Self {
        PreCommittedKeyValueStore {
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
