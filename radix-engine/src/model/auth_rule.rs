use crate::errors::RuntimeError;
use crate::errors::RuntimeError::NotAuthorized;
use crate::model::AuthRule::OneOf;
use crate::model::Proof;
use sbor::*;
use scrypto::prelude::{NonFungibleAddress, ResourceDefId};
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;

/// Authorization Rule
#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub enum AuthRule {
    NonFungible(NonFungibleAddress),
    AnyOfResource(ResourceDefId),
    OneOf(Vec<AuthRule>),
    Public,
}

impl From<scrypto::resource::AuthRule> for AuthRule {
    fn from(auth_rule: scrypto::prelude::AuthRule) -> Self {
        match auth_rule {
            ::scrypto::resource::AuthRule::NonFungible(addr) => AuthRule::NonFungible(addr),
            ::scrypto::resource::AuthRule::OneOf(auth_rules) => AuthRule::OneOf(auth_rules.into_iter().map(AuthRule::from).collect())
        }
    }
}

impl AuthRule {
    pub fn or(self, other: AuthRule) -> Self {
        match self {
            AuthRule::NonFungible(_) => OneOf(vec![self, other]),
            AuthRule::AnyOfResource(_) => OneOf(vec![self, other]),
            AuthRule::OneOf(mut rules) => {
                rules.push(other);
                OneOf(rules)
            }
            AuthRule::Public => self,
        }
    }

    pub fn check(&self, proofs_vector: &[&[Proof]]) -> Result<(), RuntimeError> {
        match self {
            AuthRule::NonFungible(non_fungible_address) => {
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
            }
            AuthRule::AnyOfResource(resource_def_id) => {
                for proofs in proofs_vector {
                    for p in proofs.iter() {
                        let proof_resource_def_id = p.resource_def_id();
                        if proof_resource_def_id == *resource_def_id {
                            return Ok(());
                        }
                    }
                }

                Err(NotAuthorized)
            }
            AuthRule::OneOf(rules) => {
                for rule in rules {
                    if rule.check(proofs_vector).is_ok() {
                        return Ok(());
                    }
                }

                Err(NotAuthorized)
            }
            AuthRule::Public => Ok(()),
        }
    }
}
