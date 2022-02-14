use sbor::*;
use scrypto::engine::types::*;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;

/// A component is an instance of blueprint.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Component {
    package_ref: PackageRef,
    blueprint_name: String,
    state: Vec<u8>,
}

impl Component {
    pub fn new(package_ref: PackageRef, blueprint_name: String, state: Vec<u8>) -> Self {
        Self {
            package_ref,
            blueprint_name,
            state,
        }
    }

    pub fn package_ref(&self) -> PackageRef {
        self.package_ref
    }

    pub fn blueprint_name(&self) -> &str {
        &self.blueprint_name
    }

    pub fn state(&self) -> &[u8] {
        &self.state
    }

    pub fn set_state(&mut self, new_state: Vec<u8>) {
        self.state = new_state;
    }
}
