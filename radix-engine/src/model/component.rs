use sbor::*;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone)]
pub enum ComponentError {
    UnauthorizedAccess,
}

/// A component is an instance of blueprint.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Component {
    package: Address,
    name: String,
    state: Vec<u8>,
    auth: Address,
}

impl Component {
    pub fn new(package: Address, name: String, state: Vec<u8>) -> Self {
        assert!(package.is_package());

        Self {
            package,
            name,
            state,
            auth: package,
        }
    }

    pub fn package(&self) -> Address {
        self.package
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn state(&self, auth: Vec<Address>) -> Result<&[u8], ComponentError> {
        if auth.contains(&self.auth) {
            Ok(&self.state)
        } else {
            Err(ComponentError::UnauthorizedAccess)
        }
    }

    pub fn set_state(
        &mut self,
        new_state: Vec<u8>,
        auth: Vec<Address>,
    ) -> Result<(), ComponentError> {
        if auth.contains(&self.auth) {
            self.state = new_state;
            Ok(())
        } else {
            Err(ComponentError::UnauthorizedAccess)
        }
    }
}
