use sbor::*;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

use crate::model::Actor;

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone)]
pub enum ComponentError {
    UnauthorizedAccess,
}

/// A component is an instance of blueprint.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Component {
    package_address: Address,
    blueprint_name: String,
    state: Vec<u8>,
    authority: Address,
}

impl Component {
    pub fn new(package_address: Address, blueprint_name: String, state: Vec<u8>) -> Self {
        Self {
            package_address,
            blueprint_name,
            state,
            authority: package_address,
        }
    }

    pub fn package_address(&self) -> Address {
        self.package_address
    }

    pub fn blueprint_name(&self) -> &str {
        &self.blueprint_name
    }

    pub fn state(&self, actor: Actor) -> Result<&[u8], ComponentError> {
        if actor.check(self.authority) {
            Ok(&self.state)
        } else {
            Err(ComponentError::UnauthorizedAccess)
        }
    }

    pub fn set_state(&mut self, new_state: Vec<u8>, actor: Actor) -> Result<(), ComponentError> {
        if actor.check(self.authority) {
            self.state = new_state;
            Ok(())
        } else {
            Err(ComponentError::UnauthorizedAccess)
        }
    }
}
