use crate::errors::RuntimeError;
use crate::errors::RuntimeError::NotAuthorized;
use crate::model::Proof;
use sbor::*;
use scrypto::prelude::ProofRule;

/// Snode which verifies authorization of a method call
#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub enum MethodAuthorization {
    Protected(ProofRule),
    Public,
    Private,
    Unsupported,
}

impl MethodAuthorization {
    pub fn check(&self, proofs_vector: &[&[Proof]]) -> Result<(), RuntimeError> {
        match self {
            MethodAuthorization::Protected(rule) => Self::check_proof_rule(rule, proofs_vector),
            MethodAuthorization::Public => Ok(()),
            MethodAuthorization::Private => Err(RuntimeError::NotAuthorized),
            MethodAuthorization::Unsupported => Err(RuntimeError::UnsupportedMethod),
        }
    }

    fn check_proof_rule(
        proof_rule: &ProofRule,
        proofs_vector: &[&[Proof]],
    ) -> Result<(), RuntimeError> {
        match proof_rule {
            ProofRule::NonFungible(non_fungible_address) => {
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
            ProofRule::AnyOfResource(resource_def_id) => {
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
            ProofRule::SomeOfResource(amount, resource_def_id) => {
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
            ProofRule::AllOf(rules) => {
                for rule in rules {
                    if Self::check_proof_rule(rule, proofs_vector).is_err() {
                        return Err(NotAuthorized);
                    }
                }

                Ok(())
            }
            ProofRule::OneOf(rules) => {
                for rule in rules {
                    if Self::check_proof_rule(rule, proofs_vector).is_ok() {
                        return Ok(());
                    }
                }

                Err(NotAuthorized)
            }
            ProofRule::CountOf { count, rules } => {
                let mut left = count.clone();
                for rule in rules {
                    if Self::check_proof_rule(rule, proofs_vector).is_ok() {
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
