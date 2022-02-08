use sbor::*;
use scrypto::rust::collections::*;
use scrypto::rust::vec::Vec;
use scrypto::types::Address;

/// A key-value map where keys and values are lazily loaded on-demand.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct LazyMap {
    map: HashMap<Vec<u8>, Vec<u8>>,
    authority: Address,
}

impl LazyMap {
    pub fn new(authority: Address) -> Self {
        Self {
            map: HashMap::new(),
            authority,
        }
    }

    // for diagnosis purpose only
    pub fn map(&self) -> &HashMap<Vec<u8>, Vec<u8>> {
        &self.map
    }

    pub fn get_entry(&self, key: &[u8]) -> Option<&[u8]> {
        self.map.get(key).map(|e| e.as_slice())
    }

    pub fn set_entry(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.map.insert(key, value);
    }
}
