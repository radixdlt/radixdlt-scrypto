use scrypto::engine::types::*;
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::vec::Vec;

use crate::model::{
    Proof, ProofError,
};

#[derive(Debug, Clone, PartialEq)]
pub enum AuthZoneError {
    EmptyAuthZone,
    ProofError(ProofError)
}

/// A transient resource container.
#[derive(Debug)]
pub struct AuthZone {
    pub proofs: Vec<Proof>,
}

impl AuthZone {
    pub fn new() -> Self {
        Self {
            proofs: Vec::new()
        }
    }

    pub fn pop(&mut self) -> Result<Proof, AuthZoneError> {
        if self.proofs.is_empty() {
            return Err(AuthZoneError::EmptyAuthZone);
        }

        Ok(self.proofs.remove(self.proofs.len() - 1))
    }

    pub fn push(&mut self, proof: Proof) {
        self.proofs.push(proof);
    }

    pub fn clear(&mut self) {
        loop {
            if let Some(proof) = self.proofs.pop() {
                proof.drop();
            } else {
                break;
            }
        }
    }

    pub fn create_proof(&self, resource_address: ResourceAddress, resource_type: ResourceType) -> Result<Proof, AuthZoneError> {
        Proof::compose(&self.proofs, resource_address, resource_type)
            .map_err(AuthZoneError::ProofError)
    }

    pub fn create_proof_by_amount(&self, amount:Decimal, resource_address: ResourceAddress, resource_type: ResourceType) -> Result<Proof, AuthZoneError> {
        Proof::compose_by_amount(&self.proofs, amount, resource_address, resource_type)
            .map_err(AuthZoneError::ProofError)
    }


    pub fn create_proof_by_ids(&self, ids: &BTreeSet<NonFungibleId>, resource_address: ResourceAddress, resource_type: ResourceType) -> Result<Proof, AuthZoneError> {
        Proof::compose_by_ids(&self.proofs, ids, resource_address, resource_type)
            .map_err(AuthZoneError::ProofError)
    }
}