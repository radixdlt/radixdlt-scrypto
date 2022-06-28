use sbor::rust::collections::BTreeSet;
use sbor::rust::vec::Vec;
use sbor::DecodeError;
use scrypto::buffer::scrypto_decode;
use scrypto::engine::types::*;
use scrypto::resource::{
    AuthZoneClearInput, AuthZoneCreateProofByAmountInput, AuthZoneCreateProofByIdsInput,
    AuthZoneCreateProofInput, AuthZonePopInput, AuthZonePushInput,
};
use scrypto::values::ScryptoValue;

use crate::engine::SystemApi;
use crate::model::AuthZoneError::InvalidMethod;
use crate::model::{Proof, ProofError};
use crate::wasm::*;

#[derive(Debug, Clone, PartialEq)]
pub enum AuthZoneError {
    EmptyAuthZone,
    ProofError(ProofError),
    CouldNotCreateProof,
    InvalidRequestData(DecodeError),
    CouldNotGetProof,
    CouldNotGetResource,
    NoMethodSpecified,
    InvalidMethod,
}

/// A transient resource container.
#[derive(Debug)]
pub struct AuthZone {
    pub proofs: Vec<Proof>,
}

impl AuthZone {
    pub fn new_with_proofs(proofs: Vec<Proof>) -> Self {
        Self { proofs }
    }

    pub fn new() -> Self {
        Self { proofs: Vec::new() }
    }

    fn pop(&mut self) -> Result<Proof, AuthZoneError> {
        if self.proofs.is_empty() {
            return Err(AuthZoneError::EmptyAuthZone);
        }

        Ok(self.proofs.remove(self.proofs.len() - 1))
    }

    fn push(&mut self, proof: Proof) {
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

    fn create_proof(
        &self,
        resource_address: ResourceAddress,
        resource_type: ResourceType,
    ) -> Result<Proof, AuthZoneError> {
        Proof::compose(&self.proofs, resource_address, resource_type)
            .map_err(AuthZoneError::ProofError)
    }

    fn create_proof_by_amount(
        &self,
        amount: Decimal,
        resource_address: ResourceAddress,
        resource_type: ResourceType,
    ) -> Result<Proof, AuthZoneError> {
        Proof::compose_by_amount(&self.proofs, amount, resource_address, resource_type)
            .map_err(AuthZoneError::ProofError)
    }

    fn create_proof_by_ids(
        &self,
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
        resource_type: ResourceType,
    ) -> Result<Proof, AuthZoneError> {
        Proof::compose_by_ids(&self.proofs, ids, resource_address, resource_type)
            .map_err(AuthZoneError::ProofError)
    }

    pub fn main<'borrowed, S: SystemApi<'borrowed, W, I>, W: WasmEngine<I>, I: WasmInstance>(
        &mut self,
        method_name: &str,
        arg: ScryptoValue,
        system_api: &mut S,
    ) -> Result<ScryptoValue, AuthZoneError> {
        match method_name {
            "pop" => {
                let _: AuthZonePopInput =
                    scrypto_decode(&arg.raw).map_err(|e| AuthZoneError::InvalidRequestData(e))?;
                let proof = self.pop()?;
                let proof_id = system_api
                    .create_proof(proof)
                    .map_err(|_| AuthZoneError::CouldNotCreateProof)?;
                Ok(ScryptoValue::from_typed(&scrypto::resource::Proof(
                    proof_id,
                )))
            }
            "push" => {
                let input: AuthZonePushInput =
                    scrypto_decode(&arg.raw).map_err(|e| AuthZoneError::InvalidRequestData(e))?;
                let mut proof: Proof = system_api
                    .take_native_value(&ValueId::Transient(TransientValueId::Proof(input.proof.0)))
                    .into();
                // FIXME: this is a hack for now until we can get snode_state into process
                // FIXME: and be able to determine which snode the proof is going into
                proof.change_to_unrestricted();

                self.push(proof);
                Ok(ScryptoValue::from_typed(&()))
            }
            "create_proof" => {
                let input: AuthZoneCreateProofInput =
                    scrypto_decode(&arg.raw).map_err(|e| AuthZoneError::InvalidRequestData(e))?;
                let resource_manager = system_api
                    .borrow_global_resource_manager(input.resource_address)
                    .map_err(|_| AuthZoneError::CouldNotGetResource)?;
                let resource_type = resource_manager.resource_type();
                let proof = self.create_proof(input.resource_address, resource_type)?;
                let proof_id = system_api
                    .create_proof(proof)
                    .map_err(|_| AuthZoneError::CouldNotCreateProof)?;
                Ok(ScryptoValue::from_typed(&scrypto::resource::Proof(
                    proof_id,
                )))
            }
            "create_proof_by_amount" => {
                let input: AuthZoneCreateProofByAmountInput =
                    scrypto_decode(&arg.raw).map_err(|e| AuthZoneError::InvalidRequestData(e))?;
                let resource_manager = system_api
                    .borrow_global_resource_manager(input.resource_address)
                    .map_err(|_| AuthZoneError::CouldNotGetResource)?;
                let resource_type = resource_manager.resource_type();
                let proof = self.create_proof_by_amount(
                    input.amount,
                    input.resource_address,
                    resource_type,
                )?;
                let proof_id = system_api
                    .create_proof(proof)
                    .map_err(|_| AuthZoneError::CouldNotCreateProof)?;
                Ok(ScryptoValue::from_typed(&scrypto::resource::Proof(
                    proof_id,
                )))
            }
            "create_proof_by_ids" => {
                let input: AuthZoneCreateProofByIdsInput =
                    scrypto_decode(&arg.raw).map_err(|e| AuthZoneError::InvalidRequestData(e))?;
                let resource_manager = system_api
                    .borrow_global_resource_manager(input.resource_address)
                    .map_err(|_| AuthZoneError::CouldNotGetResource)?;
                let resource_type = resource_manager.resource_type();
                let proof =
                    self.create_proof_by_ids(&input.ids, input.resource_address, resource_type)?;
                let proof_id = system_api
                    .create_proof(proof)
                    .map_err(|_| AuthZoneError::CouldNotCreateProof)?;
                Ok(ScryptoValue::from_typed(&scrypto::resource::Proof(
                    proof_id,
                )))
            }
            "clear" => {
                let _: AuthZoneClearInput =
                    scrypto_decode(&arg.raw).map_err(|e| AuthZoneError::InvalidRequestData(e))?;
                self.clear();
                Ok(ScryptoValue::from_typed(&()))
            }
            _ => Err(InvalidMethod),
        }
    }
}
