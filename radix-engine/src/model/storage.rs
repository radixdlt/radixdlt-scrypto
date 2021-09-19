use sbor::*;
use scrypto::rust::collections::*;
use scrypto::rust::vec::Vec;
use scrypto::types::Address;

/// A key-value storage.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Storage {
    pub owner: Address,
    pub storage: HashMap<Vec<u8>, Vec<u8>>,
}

impl Storage {
    pub fn new(owner: Address) -> Self {
        Self {
            owner,
            storage: HashMap::new(),
        }
    }

    pub fn owner(&self) -> Address {
        self.owner
    }

    pub fn get_entry(&self, key: &[u8]) -> Option<&[u8]> {
        self.storage.get(key).map(|e| e.as_slice())
    }

    pub fn set_entry(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.storage.insert(key, value);
    }
}
