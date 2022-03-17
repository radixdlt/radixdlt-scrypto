use crate::errors::RuntimeError;
use crate::errors::RuntimeError::NotAuthorized;
use crate::model::Proof;
use sbor::*;
use scrypto::prelude::{NonFungibleAddress, ResourceDefId};
use scrypto::rust::vec::Vec;
use scrypto::rust::vec;
use crate::model::AuthRule::Or;

/// Authorization Rule
#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub enum AuthRule {
    JustNonFungible(NonFungibleAddress),
    JustResource(ResourceDefId),
    Or(Vec<AuthRule>),
    All,
}

impl AuthRule {
    pub fn just_non_fungible(non_fungible_address: NonFungibleAddress) -> Self {
        AuthRule::JustNonFungible(non_fungible_address)
    }

    pub fn or(self, other: AuthRule) -> Self {
        match self {
            AuthRule::JustNonFungible(_) => Or(vec![self, other]),
            AuthRule::JustResource(_) => Or(vec![self, other]),
            AuthRule::Or(mut rules) => {
                rules.push(other);
                Or(rules)
            },
            AuthRule::All => self
        }
    }

    pub fn check(&self, proofs_vector: &[&[Proof]]) -> Result<(), RuntimeError> {
        match self {
            AuthRule::JustNonFungible(non_fungible_address) => {
                for proofs in proofs_vector {
                    for p in proofs.iter() {
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

                Err(NotAuthorized)
            },
            AuthRule::JustResource(resource_def_id) => {
                for proofs in proofs_vector {
                    for p in proofs.iter() {
                        let proof_resource_def_id = p.resource_def_id();
                        if proof_resource_def_id == *resource_def_id {
                            return Ok(());
                        }
                    }
                }

                Err(NotAuthorized)
            },
            AuthRule::Or(rules) => {
                for rule in rules {
                    if rule.check(proofs_vector).is_ok() {
                        return Ok(());
                    }
                }

                Err(NotAuthorized)
            },
            AuthRule::All => Ok(())
        }
   }
}
