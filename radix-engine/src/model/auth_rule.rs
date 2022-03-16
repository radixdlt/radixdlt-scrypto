use crate::errors::RuntimeError;
use crate::errors::RuntimeError::NotAuthorized;
use crate::model::Proof;
use sbor::*;
use scrypto::prelude::{NonFungibleAddress, ResourceDefId};
use scrypto::rust::vec::Vec;

/// Authorization Rule
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum AuthRule {
    JustNonFungible(NonFungibleAddress),
    JustResource(ResourceDefId),
}

impl AuthRule {
    pub fn just_non_fungible(non_fungible_address: NonFungibleAddress) -> Self {
        AuthRule::JustNonFungible(non_fungible_address)
    }

    pub fn check(&self, proofs_vector: Vec<&[Proof]>) -> Result<(), RuntimeError> {
        match self {
            AuthRule::JustNonFungible(non_fungible_address) => {
                for proofs in proofs_vector {
                    for p in proofs {
                        let proof_resource_def_id = p.resource_def_id();
                        if proof_resource_def_id == non_fungible_address.resource_def_id()
                            && match p.total_amount().as_non_fungible_ids() {
                                Some(ids) => ids.contains(&non_fungible_address.non_fungible_id()),
                                None => false,
                            }
                        {
                            return Ok(());
                        }
                    }
                }

                return Err(NotAuthorized);
            },
            AuthRule::JustResource(resource_def_id) => {
                for proofs in proofs_vector {
                    for p in proofs {
                        let proof_resource_def_id = p.resource_def_id();
                        if proof_resource_def_id == *resource_def_id {
                            return Ok(());
                        }
                    }
                }

                return Err(NotAuthorized);
            }
        }
   }
}
