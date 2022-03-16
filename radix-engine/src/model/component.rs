use crate::errors::RuntimeError;
use crate::errors::RuntimeError::NotAuthorized;
use crate::model::Proof;
use sbor::*;
use scrypto::engine::types::*;
use scrypto::prelude::NonFungibleAddress;
use scrypto::rust::collections::*;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;

/// A component is an instance of blueprint.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Component {
    package_id: PackageId,
    blueprint_name: String,
    state: Vec<u8>,
    sys_auth: HashMap<String, NonFungibleAddress>,
}

impl Component {
    pub fn new(
        package_id: PackageId,
        blueprint_name: String,
        state: Vec<u8>,
        sys_auth: HashMap<String, NonFungibleAddress>,
    ) -> Self {
        Self {
            package_id,
            blueprint_name,
            state,
            sys_auth,
        }
    }

    pub fn check_auth(&self, function: &str, proofs: &[Proof]) -> Result<(), RuntimeError> {
        if let Some(auth_address) = self.sys_auth.get(function) {
            if !proofs.iter().any(|p| {
                p.resource_def_id() == auth_address.resource_def_id()
                    && match p.total_amount().as_non_fungible_ids() {
                        Ok(ids) => ids.contains(&auth_address.non_fungible_id()),
                        Err(_) => false,
                    }
            }) {
                return Err(NotAuthorized);
            }
        }

        Ok(())
    }

    pub fn package_id(&self) -> PackageId {
        self.package_id
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
