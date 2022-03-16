use crate::errors::RuntimeError;
use crate::errors::RuntimeError::NotAuthorized;
use crate::model::Proof;
use sbor::*;
use scrypto::prelude::NonFungibleAddress;

/// Authorization Rule
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct AuthRule {
    non_fungible_address: NonFungibleAddress,
}

impl AuthRule {
    pub fn new(non_fungible_address: NonFungibleAddress) -> Self {
        Self {
            non_fungible_address
        }
    }

    pub fn non_fungible_address(&self) -> &NonFungibleAddress {
        &self.non_fungible_address
    }

    pub fn check(&self, proofs: &[Proof]) -> Result<(), RuntimeError> {
        if !proofs.iter().any(|p| {
            p.resource_def_id() == self.non_fungible_address.resource_def_id()
                && match p.total_amount().as_non_fungible_ids() {
                Some(ids) => ids.contains(&self.non_fungible_address.non_fungible_id()),
                None => false,
            }
        }) {
            return Err(NotAuthorized);
        }

        Ok(())
    }
}
