use crate::errors::RuntimeError;
use crate::errors::RuntimeError::NotAuthorized;
use crate::model::Proof;
use sbor::*;
use scrypto::math::Decimal;
use scrypto::prelude::{NonFungibleAddress, ResourceDefId};
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub enum HardProofRuleResource {
    NonFungible(NonFungibleAddress),
    Resource(ResourceDefId),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub enum HardProofRule {
    This(HardProofRuleResource),
    SomeOfResource(Decimal, ResourceDefId),
    AllOf(Vec<HardProofRule>),
    OneOf(Vec<HardProofRule>),
    CountOf {
        count: u8,
        rules: Vec<HardProofRule>,
    },
}

impl From<NonFungibleAddress> for HardProofRule {
    fn from(non_fungible_address: NonFungibleAddress) -> Self {
        HardProofRule::This(HardProofRuleResource::NonFungible(non_fungible_address.into()))
    }
}

impl From<ResourceDefId> for HardProofRule {
    fn from(resource_def_id: ResourceDefId) -> Self {
        HardProofRule::This(HardProofRuleResource::Resource(resource_def_id.into()))
    }
}

impl HardProofRule {
    pub fn or(self, other: HardProofRule) -> Self {
        match self {
            HardProofRule::This(_) => HardProofRule::OneOf(vec![self, other]),
            HardProofRule::SomeOfResource(_, _) => HardProofRule::OneOf(vec![self, other]),
            HardProofRule::AllOf(_) => HardProofRule::OneOf(vec![self, other]),
            HardProofRule::OneOf(mut rules) => {
                rules.push(other);
                HardProofRule::OneOf(rules)
            }
            HardProofRule::CountOf { count: _, rules: _ } => {
                HardProofRule::OneOf(vec![self, other])
            }
        }
    }

    pub fn check(&self, proofs_vector: &[&[Proof]]) -> Result<(), RuntimeError> {
        match self {
            HardProofRule::This(proof_rule_resource) => match proof_rule_resource {
                HardProofRuleResource::NonFungible(non_fungible_address) => {
                    for proofs in proofs_vector {
                        for p in proofs.iter() {
                            let proof_resource_def_id = p.resource_def_id();
                            if proof_resource_def_id == non_fungible_address.resource_def_id()
                                && match p.total_ids() {
                                    Ok(ids) => {
                                        ids.contains(&non_fungible_address.non_fungible_id())
                                    }
                                    Err(_) => false,
                                }
                            {
                                return Ok(());
                            }
                        }
                    }

                    Err(NotAuthorized)
                }
                HardProofRuleResource::Resource(resource_def_id) => {
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
            },
            HardProofRule::SomeOfResource(amount, resource_def_id) => {
                for proofs in proofs_vector {
                    for p in proofs.iter() {
                        let proof_resource_def_id = p.resource_def_id();
                        if proof_resource_def_id == *resource_def_id && p.total_amount() >= *amount
                        {
                            return Ok(());
                        }
                    }
                }

                Err(NotAuthorized)
            }
            HardProofRule::AllOf(rules) => {
                for rule in rules {
                    if rule.check(proofs_vector).is_err() {
                        return Err(NotAuthorized);
                    }
                }

                Ok(())
            }
            HardProofRule::OneOf(rules) => {
                for rule in rules {
                    if rule.check(proofs_vector).is_ok() {
                        return Ok(());
                    }
                }

                Err(NotAuthorized)
            }
            HardProofRule::CountOf { count, rules } => {
                let mut left = count.clone();
                for rule in rules {
                    if rule.check(proofs_vector).is_ok() {
                        left -= 1;
                        if left == 0 {
                            return Ok(());
                        }
                    }
                }

                Err(NotAuthorized)
            }
        }
    }
}

/// Snode which verifies authorization of a method call
#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub enum MethodAuthorization {
    Protected(HardProofRule),
    Public,
    Private,
    Unsupported,
}

impl MethodAuthorization {
    pub fn check(&self, proofs_vector: &[&[Proof]]) -> Result<(), RuntimeError> {
        match self {
            MethodAuthorization::Protected(rule) => rule.check(proofs_vector),
            MethodAuthorization::Public => Ok(()),
            MethodAuthorization::Private => Err(RuntimeError::NotAuthorized),
            MethodAuthorization::Unsupported => Err(RuntimeError::UnsupportedMethod),
        }
    }
}
