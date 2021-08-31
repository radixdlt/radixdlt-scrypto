use sbor::*;
use scrypto::rust::collections::*;
use scrypto::rust::vec::Vec;

#[derive(Debug, Clone, Encode, Decode)]
pub struct Map {
    map: HashMap<Vec<u8>, Vec<u8>>,
}

impl Map {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn get_entry(&self, key: &Vec<u8>) -> Option<&Vec<u8>> {
        self.map.get(key)
    }

    pub fn set_entry(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.map.insert(key, value);
    }
}
