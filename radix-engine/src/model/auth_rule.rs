use crate::errors::RuntimeError;
use crate::errors::RuntimeError::NotAuthorized;
use crate::model::Proof;
use sbor::*;
use scrypto::prelude::{NonFungibleAddress, ResourceDefId};
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub enum Rule {
    NonFungible(NonFungibleAddress),
    AnyOfResource(ResourceDefId),
    OneOf(Vec<Rule>),
}

impl Rule {
    pub fn or(self, other: Rule) -> Self {
        match self {
            Rule::NonFungible(_) => Rule::OneOf(vec![self, other]),
            Rule::AnyOfResource(_) => Rule::OneOf(vec![self, other]),
            Rule::OneOf(mut rules) => {
                rules.push(other);
                Rule::OneOf(rules)
            }
        }
    }

    pub fn check(&self, proofs_vector: &[&[Proof]]) -> Result<(), RuntimeError> {
        match self {
            Rule::NonFungible(non_fungible_address) => {
                for proofs in proofs_vector {
                    for p in proofs.iter() {
                        let proof_resource_def_id = p.resource_def_id();
                        if proof_resource_def_id == non_fungible_address.resource_def_id()
                            && match p.total_ids() {
                                Ok(ids) => ids.contains(&non_fungible_address.non_fungible_id()),
                                Err(_) => false,
                            }
                        {
                            return Ok(());
                        }
                    }
                }

                Err(NotAuthorized)
            }
            Rule::AnyOfResource(resource_def_id) => {
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
            Rule::OneOf(rules) => {
                for rule in rules {
                    if rule.check(proofs_vector).is_ok() {
                        return Ok(());
                    }
                }

                Err(NotAuthorized)
            }
        }
    }
}

impl From<scrypto::resource::AuthRule> for Rule {
    fn from(auth_rule: scrypto::prelude::AuthRule) -> Self {
        match auth_rule {
            ::scrypto::resource::AuthRule::NonFungible(addr) => Rule::NonFungible(addr),
            ::scrypto::resource::AuthRule::OneOf(auth_rules) => {
                Rule::OneOf(auth_rules.into_iter().map(Rule::from).collect())
            }
        }
    }
}

/// Authorization Rule
#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub enum AuthRule {
    Protected(Rule),
    Public,
    Private,
    Unsupported,
}

impl AuthRule {
    pub fn check(&self, proofs_vector: &[&[Proof]]) -> Result<(), RuntimeError> {
        match self {
            AuthRule::Protected(rule) => rule.check(proofs_vector),
            AuthRule::Public => Ok(()),
            AuthRule::Private => Err(RuntimeError::NotAuthorized),
            AuthRule::Unsupported => Err(RuntimeError::UnsupportedMethod),
        }
    }
}
