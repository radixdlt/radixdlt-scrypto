use sbor::DecodeError;
use scrypto::engine::types::*;
use scrypto::prelude::scrypto_decode;
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::vec::Vec;
use scrypto::rust::string::String;
use scrypto::rust::string::ToString;
use scrypto::values::ScryptoValue;
use crate::engine::SystemApi;

use crate::model::{Proof, ProofError, ResourceManager};

#[derive(Debug, Clone, PartialEq)]
pub enum AuthZoneError {
    EmptyAuthZone,
    ProofError(ProofError),
    CouldNotCreateProof,
    MethodNotFound(String),
    InvalidRequestData(DecodeError),
    CouldNotGetProof,
    CouldNotGetResource,
}

/// A transient resource container.
#[derive(Debug)]
pub struct AuthZone {
    pub proofs: Vec<Proof>,
}

impl AuthZone {
    pub fn new_with_proofs(proofs: Vec<Proof>) -> Self {
        Self {
            proofs
        }
    }

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

    fn clear(&mut self) {
        loop {
            if let Some(proof) = self.proofs.pop() {
                proof.drop();
            } else {
                break;
            }
        }
    }

    fn create_proof(&self, resource_address: ResourceAddress, resource_type: ResourceType) -> Result<Proof, AuthZoneError> {
        Proof::compose(&self.proofs, resource_address, resource_type)
            .map_err(AuthZoneError::ProofError)
    }

    fn create_proof_by_amount(&self, amount:Decimal, resource_address: ResourceAddress, resource_type: ResourceType) -> Result<Proof, AuthZoneError> {
        Proof::compose_by_amount(&self.proofs, amount, resource_address, resource_type)
            .map_err(AuthZoneError::ProofError)
    }


    fn create_proof_by_ids(&self, ids: &BTreeSet<NonFungibleId>, resource_address: ResourceAddress, resource_type: ResourceType) -> Result<Proof, AuthZoneError> {
        Proof::compose_by_ids(&self.proofs, ids, resource_address, resource_type)
            .map_err(AuthZoneError::ProofError)
    }

    pub fn main<S: SystemApi>(
        &mut self,
        function: &str,
        args: Vec<ScryptoValue>,
        system_api: &mut S,
    ) -> Result<ScryptoValue, AuthZoneError> {
        match function {
            "clear" => {
                self.clear();
                Ok(ScryptoValue::from_value(&()))
            }
            "pop" => {
                let proof = self.pop()?;
                let proof_id = system_api.create_proof(proof).map_err(|_| AuthZoneError::CouldNotCreateProof)?;
                Ok(ScryptoValue::from_value(&scrypto::resource::Proof(proof_id)))
            }
            "push" => {
                let proof_id: scrypto::resource::Proof =
                    scrypto_decode(&args[0].raw).map_err(|e| AuthZoneError::InvalidRequestData(e))?;
                let mut proof = system_api.take_proof(proof_id.0).map_err(|_| AuthZoneError::CouldNotGetProof)?;
                // FIXME: this is a hack for now until we can get snode_state into process
                // FIXME: and be able to determine which snode the proof is going into
                proof.change_to_unrestricted();

                self.push(proof);
                Ok(ScryptoValue::from_value(&()))
            }
            "create_proof" => {
                let resource_address = scrypto_decode(&args[0].raw).map_err(|e| AuthZoneError::InvalidRequestData(e))?;
                let resource_manager: ResourceManager = system_api.borrow_global_mut_resource_manager(resource_address).map_err(|_| AuthZoneError::CouldNotGetResource)?;
                let resource_type = resource_manager.resource_type();
                system_api.return_borrowed_global_resource_manager(resource_address, resource_manager);
                let proof = self.create_proof(resource_address, resource_type)?;
                let proof_id = system_api.create_proof(proof).map_err(|_| AuthZoneError::CouldNotCreateProof)?;
                Ok(ScryptoValue::from_value(&scrypto::resource::Proof(proof_id)))
            }
            "create_proof_by_amount" => {
                let amount = scrypto_decode(&args[0].raw).map_err(|e| AuthZoneError::InvalidRequestData(e))?;
                let resource_address = scrypto_decode(&args[1].raw).map_err(|e| AuthZoneError::InvalidRequestData(e))?;
                let resource_manager: ResourceManager = system_api.borrow_global_mut_resource_manager(resource_address).map_err(|_| AuthZoneError::CouldNotGetResource)?;
                let resource_type = resource_manager.resource_type();
                system_api.return_borrowed_global_resource_manager(resource_address, resource_manager);
                let proof = self.create_proof_by_amount(amount, resource_address, resource_type)?;
                let proof_id = system_api.create_proof(proof).map_err(|_| AuthZoneError::CouldNotCreateProof)?;
                Ok(ScryptoValue::from_value(&scrypto::resource::Proof(proof_id)))
            }
            "create_proof_by_ids" => {
                let ids = scrypto_decode(&args[0].raw).map_err(|e| AuthZoneError::InvalidRequestData(e))?;
                let resource_address = scrypto_decode(&args[1].raw).map_err(|e| AuthZoneError::InvalidRequestData(e))?;
                let resource_manager: ResourceManager = system_api.borrow_global_mut_resource_manager(resource_address).map_err(|_| AuthZoneError::CouldNotGetResource)?;
                let resource_type = resource_manager.resource_type();
                system_api.return_borrowed_global_resource_manager(resource_address, resource_manager);
                let proof = self.create_proof_by_ids(&ids, resource_address, resource_type)?;
                let proof_id = system_api.create_proof(proof).map_err(|_| AuthZoneError::CouldNotCreateProof)?;
                Ok(ScryptoValue::from_value(&scrypto::resource::Proof(proof_id)))
            }
            _ => Err(AuthZoneError::MethodNotFound(function.to_string())),
        }
    }
}