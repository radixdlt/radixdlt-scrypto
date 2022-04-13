use scrypto::engine::types::*;
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::vec::Vec;
use scrypto::rust::string::String;
use scrypto::rust::string::ToString;
use scrypto::values::ScryptoValue;
use crate::engine::SystemApi;

use crate::model::{
    Proof, ProofError,
};

#[derive(Debug, Clone, PartialEq)]
pub enum AuthZoneError {
    EmptyAuthZone,
    ProofError(ProofError),
    CouldNotCreateProof,
    MethodNotFound(String),
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

    pub fn main<S: SystemApi>(
        &mut self,
        function: &str,
        _args: Vec<ScryptoValue>,
        system_api: &mut S,
    ) -> Result<ScryptoValue, AuthZoneError> {
        match function {
            "pop" => {
                let proof = self.pop()?;
                let proof_id = system_api.create_proof(proof).map_err(|_| AuthZoneError::CouldNotCreateProof)?;
                Ok(ScryptoValue::from_value(&scrypto::resource::Proof(proof_id)))
            }
            _ => Err(AuthZoneError::MethodNotFound(function.to_string())),
        }
    }
}