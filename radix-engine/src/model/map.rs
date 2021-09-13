use sbor::*;
use scrypto::rust::collections::*;
use scrypto::rust::vec::Vec;
use scrypto::types::Address;

/// A key-value storage.
#[derive(Debug, Clone, Encode, Decode)]
pub struct Map {
    pub owner: Address,
    pub map: HashMap<Vec<u8>, Vec<u8>>,
}

impl Map {
    pub fn new(owner: Address) -> Self {
        Self {
            owner,
            map: HashMap::new(),
        }
    }

    pub fn owner(&self) -> Address {
        self.owner
    }

    pub fn get_entry(&self, key: &[u8]) -> Option<&[u8]> {
        self.map.get(key).map(|e| e.as_slice())
    }

    pub fn set_entry(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.map.insert(key, value);
    }
}
