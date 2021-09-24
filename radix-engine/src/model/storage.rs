use sbor::*;
use scrypto::rust::collections::*;
use scrypto::rust::vec::Vec;
use scrypto::types::Address;

/// A key-value storage.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Storage {
    pub storage: HashMap<Vec<u8>, Vec<u8>>,
    pub auth: Address,
}

impl Storage {
    pub fn new(auth: Address) -> Self {
        Self {
            storage: HashMap::new(),
            auth,
        }
    }

    pub fn auth(&self) -> Address {
        self.auth
    }

    pub fn get_entry(&self, key: &[u8]) -> Option<&[u8]> {
        self.storage.get(key).map(|e| e.as_slice())
    }

    pub fn set_entry(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.storage.insert(key, value);
    }
}
