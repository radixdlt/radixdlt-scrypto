use sbor::*;
use scrypto::rust::collections::*;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

/// A component is an instance of blueprint.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Component {
    package: Address,
    blueprint: String,
    state: Vec<u8>,
    map: HashMap<Vec<u8>, Vec<u8>>,
}

impl Component {
    pub fn new(package: Address, blueprint: String, state: Vec<u8>) -> Self {
        Self {
            package,
            blueprint,
            state,
            map: HashMap::new(),
        }
    }

    pub fn package(&self) -> Address {
        self.package
    }

    pub fn blueprint(&self) -> &str {
        &self.blueprint
    }

    pub fn state(&self) -> &[u8] {
        &self.state
    }

    pub fn set_state(&mut self, new_state: Vec<u8>) {
        self.state = new_state;
    }

    pub fn map_entry(&self, key: &[u8]) -> Option<&[u8]> {
        self.map.get(key).map(|e| e.as_slice())
    }

    pub fn set_map_entry(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.map.insert(key, value);
    }
}
