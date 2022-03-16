use crate::errors::RuntimeError;
use crate::errors::RuntimeError::NotAuthorized;
use crate::model::Proof;
use sbor::*;
use scrypto::prelude::NonFungibleAddress;

/// Authorization Rule
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum AuthRule {
    Just(NonFungibleAddress),
}

impl AuthRule {
    pub fn just(non_fungible_address: NonFungibleAddress) -> Self {
        AuthRule::Just(non_fungible_address)
    }

    pub fn check(&self, proofs: &[Proof]) -> Result<(), RuntimeError> {
        match self {
            AuthRule::Just(non_fungible_address) => {
                if !proofs.iter().any(|p| {
                    p.resource_def_id() == non_fungible_address.resource_def_id()
                        && match p.total_amount().as_non_fungible_ids() {
                        Some(ids) => ids.contains(&non_fungible_address.non_fungible_id()),
                        None => false,
                    }
                }) {
                    return Err(NotAuthorized);
                }

                Ok(())
            }
        }
   }
}
