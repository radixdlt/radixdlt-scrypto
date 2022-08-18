use crate::engine::{HeapRENode, SystemApi};
use crate::fee::FeeReserve;
use crate::fee::FeeReserveError;
use crate::model::{Proof, ProofError};
use crate::types::*;
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
    CostingError(FeeReserveError),
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

    pub fn clear(&mut self) {
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

    pub fn main<'s, Y: SystemApi<'s, W, I, C>, W: WasmEngine<I>, I: WasmInstance, C: FeeReserve>(
        auth_zone_frame_id: usize,
        auth_zone_fn: AuthZoneFnIdentifier,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, AuthZoneError> {
        match auth_zone_fn {
            AuthZoneFnIdentifier::Pop => {
                let _: AuthZonePopInput =
                    scrypto_decode(&args.raw).map_err(|e| AuthZoneError::InvalidRequestData(e))?;
                let auth_zone = system_api.auth_zone(auth_zone_frame_id);
                let proof = auth_zone.pop()?;
                let proof_id = system_api
                    .node_create(HeapRENode::Proof(proof))
                    .unwrap()
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Proof(
                    proof_id,
                )))
            }
            AuthZoneFnIdentifier::Push => {
                let input: AuthZonePushInput =
                    scrypto_decode(&args.raw).map_err(|e| AuthZoneError::InvalidRequestData(e))?;
                let mut proof: Proof = system_api
                    .node_drop(&RENodeId::Proof(input.proof.0))
                    .map_err(AuthZoneError::CostingError)?
                    .into();
                proof.change_to_unrestricted();

                let auth_zone = system_api.auth_zone(auth_zone_frame_id);
                auth_zone.push(proof);
                Ok(ScryptoValue::from_typed(&()))
            }
            AuthZoneFnIdentifier::CreateProof => {
                let input: AuthZoneCreateProofInput =
                    scrypto_decode(&args.raw).map_err(|e| AuthZoneError::InvalidRequestData(e))?;
                let resource_type = {
                    let value = system_api
                        .borrow_node(&RENodeId::ResourceManager(input.resource_address))
                        .map_err(AuthZoneError::CostingError)?;
                    let resource_manager = value.resource_manager();
                    resource_manager.resource_type()
                };
                let auth_zone = system_api.auth_zone(auth_zone_frame_id);
                let proof = auth_zone.create_proof(input.resource_address, resource_type)?;
                let proof_id = system_api
                    .node_create(HeapRENode::Proof(proof))
                    .unwrap()
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Proof(
                    proof_id,
                )))
            }
            AuthZoneFnIdentifier::CreateProofByAmount => {
                let input: AuthZoneCreateProofByAmountInput =
                    scrypto_decode(&args.raw).map_err(|e| AuthZoneError::InvalidRequestData(e))?;
                let resource_type = {
                    let value = system_api
                        .borrow_node(&RENodeId::ResourceManager(input.resource_address))
                        .map_err(AuthZoneError::CostingError)?;
                    let resource_manager = value.resource_manager();
                    resource_manager.resource_type()
                };
                let auth_zone = system_api.auth_zone(auth_zone_frame_id);
                let proof = auth_zone.create_proof_by_amount(
                    input.amount,
                    input.resource_address,
                    resource_type,
                )?;
                let proof_id = system_api
                    .node_create(HeapRENode::Proof(proof))
                    .unwrap()
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Proof(
                    proof_id,
                )))
            }
            AuthZoneFnIdentifier::CreateProofByIds => {
                let input: AuthZoneCreateProofByIdsInput =
                    scrypto_decode(&args.raw).map_err(|e| AuthZoneError::InvalidRequestData(e))?;
                let resource_type = {
                    let value = system_api
                        .borrow_node(&RENodeId::ResourceManager(input.resource_address))
                        .map_err(AuthZoneError::CostingError)?;
                    let resource_manager = value.resource_manager();
                    resource_manager.resource_type()
                };
                let auth_zone = system_api.auth_zone(auth_zone_frame_id);
                let proof = auth_zone.create_proof_by_ids(
                    &input.ids,
                    input.resource_address,
                    resource_type,
                )?;
                let proof_id = system_api
                    .node_create(HeapRENode::Proof(proof))
                    .unwrap()
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Proof(
                    proof_id,
                )))
            }
            AuthZoneFnIdentifier::Clear => {
                let _: AuthZoneClearInput =
                    scrypto_decode(&args.raw).map_err(|e| AuthZoneError::InvalidRequestData(e))?;
                let auth_zone = system_api.auth_zone(auth_zone_frame_id);
                auth_zone.clear();
                Ok(ScryptoValue::from_typed(&()))
            }
        }
    }
}
