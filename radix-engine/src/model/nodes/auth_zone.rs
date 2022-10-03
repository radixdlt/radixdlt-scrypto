use crate::engine::{HeapRENode, SystemApi};
use crate::fee::FeeReserve;
use crate::model::{InvokeError, Proof, ProofError};
use crate::types::*;
use crate::wasm::*;
use scrypto::resource::AuthZoneDrainInput;

#[derive(Debug, TypeId, Encode, Decode)]
pub enum AuthZoneError {
    EmptyAuthZone,
    ProofError(ProofError),
    CouldNotCreateProof,
    InvalidRequestData(DecodeError),
    CouldNotGetProof,
    CouldNotGetResource,
    NoMethodSpecified,
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

    fn pop(&mut self) -> Result<Proof, InvokeError<AuthZoneError>> {
        if self.proofs.is_empty() {
            return Err(InvokeError::Error(AuthZoneError::EmptyAuthZone));
        }

        Ok(self.proofs.remove(self.proofs.len() - 1))
    }

    fn push(&mut self, proof: Proof) {
        self.proofs.push(proof);
    }

    pub fn drain(&mut self) -> Vec<Proof> {
        self.proofs.drain(0..).collect()
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
    ) -> Result<Proof, InvokeError<AuthZoneError>> {
        Proof::compose(&self.proofs, resource_address, resource_type)
            .map_err(|e| InvokeError::Error(AuthZoneError::ProofError(e)))
    }

    fn create_proof_by_amount(
        &self,
        amount: Decimal,
        resource_address: ResourceAddress,
        resource_type: ResourceType,
    ) -> Result<Proof, InvokeError<AuthZoneError>> {
        Proof::compose_by_amount(&self.proofs, amount, resource_address, resource_type)
            .map_err(|e| InvokeError::Error(AuthZoneError::ProofError(e)))
    }

    fn create_proof_by_ids(
        &self,
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
        resource_type: ResourceType,
    ) -> Result<Proof, InvokeError<AuthZoneError>> {
        Proof::compose_by_ids(&self.proofs, ids, resource_address, resource_type)
            .map_err(|e| InvokeError::Error(AuthZoneError::ProofError(e)))
    }

    pub fn main<'s, Y, W, I, R>(
        auth_zone_id: AuthZoneId,
        auth_zone_fn: AuthZoneFnIdentifier,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<AuthZoneError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        let rtn = match auth_zone_fn {
            AuthZoneFnIdentifier::Pop => {
                let _: AuthZonePopInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(AuthZoneError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::AuthZone(auth_zone_id))
                    .map_err(InvokeError::Downstream)?;
                let auth_zone = node_ref.auth_zone_mut();
                let proof = auth_zone.pop()?;
                let proof_id = system_api
                    .node_create(HeapRENode::Proof(proof))
                    .map_err(InvokeError::Downstream)?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Proof(
                    proof_id,
                )))
            }
            AuthZoneFnIdentifier::Push => {
                let input: AuthZonePushInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(AuthZoneError::InvalidRequestData(e)))?;
                let mut proof: Proof = system_api
                    .node_drop(&RENodeId::Proof(input.proof.0))
                    .map_err(InvokeError::Downstream)?
                    .into();
                proof.change_to_unrestricted();

                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::AuthZone(auth_zone_id))
                    .map_err(InvokeError::Downstream)?;
                let auth_zone = node_ref.auth_zone_mut();
                auth_zone.push(proof);
                Ok(ScryptoValue::from_typed(&()))
            }
            AuthZoneFnIdentifier::CreateProof => {
                let input: AuthZoneCreateProofInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(AuthZoneError::InvalidRequestData(e)))?;
                let resource_type = {
                    let mut node_ref = system_api
                        .borrow_node(&RENodeId::ResourceManager(input.resource_address))
                        .map_err(InvokeError::Downstream)?;
                    let resource_manager = node_ref.resource_manager();
                    resource_manager.resource_type()
                };
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::AuthZone(auth_zone_id))
                    .map_err(InvokeError::Downstream)?;
                let auth_zone = node_ref.auth_zone_mut();
                let proof = auth_zone.create_proof(input.resource_address, resource_type)?;
                let proof_id = system_api
                    .node_create(HeapRENode::Proof(proof))
                    .map_err(InvokeError::Downstream)?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Proof(
                    proof_id,
                )))
            }
            AuthZoneFnIdentifier::CreateProofByAmount => {
                let input: AuthZoneCreateProofByAmountInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(AuthZoneError::InvalidRequestData(e)))?;
                let resource_type = {
                    let mut node_ref = system_api
                        .borrow_node(&RENodeId::ResourceManager(input.resource_address))
                        .map_err(InvokeError::Downstream)?;
                    let resource_manager = node_ref.resource_manager();
                    resource_manager.resource_type()
                };
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::AuthZone(auth_zone_id))
                    .map_err(InvokeError::Downstream)?;
                let auth_zone = node_ref.auth_zone_mut();
                let proof = auth_zone.create_proof_by_amount(
                    input.amount,
                    input.resource_address,
                    resource_type,
                )?;
                let proof_id = system_api
                    .node_create(HeapRENode::Proof(proof))
                    .map_err(InvokeError::Downstream)?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Proof(
                    proof_id,
                )))
            }
            AuthZoneFnIdentifier::CreateProofByIds => {
                let input: AuthZoneCreateProofByIdsInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(AuthZoneError::InvalidRequestData(e)))?;
                let resource_type = {
                    let mut node_ref = system_api
                        .borrow_node(&RENodeId::ResourceManager(input.resource_address))
                        .map_err(InvokeError::Downstream)?;
                    let resource_manager = node_ref.resource_manager();
                    resource_manager.resource_type()
                };
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::AuthZone(auth_zone_id))
                    .map_err(InvokeError::Downstream)?;
                let auth_zone = node_ref.auth_zone_mut();
                let proof = auth_zone.create_proof_by_ids(
                    &input.ids,
                    input.resource_address,
                    resource_type,
                )?;
                let proof_id = system_api
                    .node_create(HeapRENode::Proof(proof))
                    .map_err(InvokeError::Downstream)?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Proof(
                    proof_id,
                )))
            }
            AuthZoneFnIdentifier::Clear => {
                let _: AuthZoneClearInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(AuthZoneError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::AuthZone(auth_zone_id))
                    .map_err(InvokeError::Downstream)?;
                let auth_zone = node_ref.auth_zone_mut();
                auth_zone.clear();
                Ok(ScryptoValue::from_typed(&()))
            }
            AuthZoneFnIdentifier::Drain => {
                let _: AuthZoneDrainInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(AuthZoneError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::AuthZone(auth_zone_id))
                    .map_err(InvokeError::Downstream)?;
                let auth_zone = node_ref.auth_zone_mut();
                let proofs = auth_zone.drain();
                let mut proof_ids: Vec<scrypto::resource::Proof> = Vec::new();
                for proof in proofs {
                    let proof_id: ProofId = system_api
                        .node_create(HeapRENode::Proof(proof))
                        .map_err(InvokeError::Downstream)?
                        .into();
                    proof_ids.push(scrypto::resource::Proof(proof_id));
                }

                Ok(ScryptoValue::from_typed(&proof_ids))
            }
        }?;

        Ok(rtn)
    }
}
