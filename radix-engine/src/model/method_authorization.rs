use crate::errors::RuntimeError;
use crate::errors::RuntimeError::NotAuthorized;
use crate::model::Proof;
use sbor::*;
use scrypto::math::Decimal;
use scrypto::prelude::{NonFungibleAddress, ResourceDefId};
use scrypto::rust::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub enum HardProofRuleResource {
    NonFungible(NonFungibleAddress),
    Resource(ResourceDefId),
}

impl HardProofRuleResource {
    pub fn proof_matches(&self, proof: &Proof) -> bool {
        match self {
            HardProofRuleResource::NonFungible(non_fungible_address) => {
                let proof_resource_def_id = proof.resource_def_id();
                proof_resource_def_id == non_fungible_address.resource_def_id() && match proof.total_ids() {
                    Ok(ids) => {
                        ids.contains(&non_fungible_address.non_fungible_id())
                    }
                    Err(_) => false,
                }
            }
            HardProofRuleResource::Resource(resource_def_id) => {
                let proof_resource_def_id = proof.resource_def_id();
                proof_resource_def_id == *resource_def_id
            }
        }
    }

    pub fn check_has_amount(&self, amount: Decimal, proofs_vector: &[&[Proof]]) -> bool {
        for proofs in proofs_vector {
            if proofs.iter().any(|p| self.proof_matches(p) && p.total_amount() >= amount) {
                return true;
            }
        }

        false
    }

    pub fn check(&self, proofs_vector: &[&[Proof]]) -> bool {
        for proofs in proofs_vector {
            if proofs.iter().any(|p| self.proof_matches(p)) {
                return true;
            }
        }

        false
    }
}

impl From<NonFungibleAddress> for HardProofRuleResource {
    fn from(non_fungible_address: NonFungibleAddress) -> Self {
        HardProofRuleResource::NonFungible(non_fungible_address)
    }
}

impl From<ResourceDefId> for HardProofRuleResource {
    fn from(resource_def_id: ResourceDefId) -> Self {
        HardProofRuleResource::Resource(resource_def_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub enum HardProofRule {
    This(HardProofRuleResource),
    SomeOfResource(Decimal, HardProofRuleResource),
    AllOf(Vec<HardProofRuleResource>),
    AnyOf(Vec<HardProofRuleResource>),
    CountOf(u8, Vec<HardProofRuleResource>),
}

impl HardProofRule {
    pub fn check(&self, proofs_vector: &[&[Proof]]) -> Result<(), RuntimeError> {
        match self {
            HardProofRule::This(resource) => {
                if resource.check(proofs_vector) {
                    Ok(())
                } else {
                    Err(NotAuthorized)
                }
            }
            HardProofRule::SomeOfResource(amount, resource) => {
                if resource.check_has_amount(*amount, proofs_vector) {
                    Ok(())
                } else {
                    Err(NotAuthorized)
                }
            }
            HardProofRule::AllOf(resources) => {
                for resource in resources {
                    if !resource.check(proofs_vector) {
                        return Err(NotAuthorized);
                    }
                }

                Ok(())
            }
            HardProofRule::AnyOf(resources) => {
                for resource in resources {
                    if resource.check(proofs_vector) {
                        return Ok(());
                    }
                }

                Err(NotAuthorized)
            }
            HardProofRule::CountOf(count, resources) => {
                let mut left = count.clone();
                for resource in resources {
                    if resource.check(proofs_vector) {
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
