use sbor::*;
use scrypto::types::*;

/// A component is an instance of blueprint.
#[derive(Debug, Clone, Encode, Decode)]
pub struct Component {
    package: Address,
    blueprint: String,
    state: Vec<u8>,
}

impl Component {
    pub fn new(package: Address, blueprint: String, state: Vec<u8>) -> Self {
        Self {
            package,
            blueprint,
            state,
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
}
