use crate::engine::{HeapRENode, LockFlags, SystemApi};
use crate::fee::FeeReserve;
use crate::model::{InvokeError, ProofError, ProofSubstate};
use crate::types::*;
use scrypto::resource::AuthZoneDrainInput;

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum AuthZoneError {
    EmptyAuthZone,
    ProofError(ProofError),
    CouldNotCreateProof,
    InvalidRequestData(DecodeError),
    CouldNotGetProof,
    CouldNotGetResource,
    NoMethodSpecified,
}

pub struct AuthZoneStack;

impl AuthZoneStack {
    pub fn method_locks(method: AuthZoneMethod) -> LockFlags {
        match method {
            AuthZoneMethod::Pop => LockFlags::MUTABLE,
            AuthZoneMethod::Push => LockFlags::MUTABLE,
            AuthZoneMethod::CreateProof => LockFlags::MUTABLE,
            AuthZoneMethod::CreateProofByAmount => LockFlags::MUTABLE,
            AuthZoneMethod::CreateProofByIds => LockFlags::MUTABLE,
            AuthZoneMethod::Clear => LockFlags::MUTABLE,
            AuthZoneMethod::Drain => LockFlags::MUTABLE,
        }
    }

    pub fn main<'s, Y, R>(
        auth_zone_id: AuthZoneId,
        method: AuthZoneMethod,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<AuthZoneError>>
    where
        Y: SystemApi<'s, R>,
        R: FeeReserve,
    {
        let node_id = RENodeId::AuthZoneStack(auth_zone_id);
        let offset = SubstateOffset::AuthZone(AuthZoneOffset::AuthZone);
        let auth_zone_handle =
            system_api.lock_substate(node_id, offset, Self::method_locks(method))?;

        let rtn = match method {
            AuthZoneMethod::Pop => {
                let _: AuthZonePopInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(AuthZoneError::InvalidRequestData(e)))?;

                let proof = {
                    let mut substate_mut = system_api.get_ref_mut(auth_zone_handle)?;
                    let mut raw_mut = substate_mut.get_raw_mut();
                    let auth_zone = raw_mut.auth_zone();
                    let proof = auth_zone.cur_auth_zone_mut().pop()?;
                    substate_mut.flush()?;
                    proof
                };

                let proof_id = system_api.node_create(HeapRENode::Proof(proof))?.into();
                ScryptoValue::from_typed(&scrypto::resource::Proof(proof_id))
            }
            AuthZoneMethod::Push => {
                let input: AuthZonePushInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(AuthZoneError::InvalidRequestData(e)))?;
                let mut proof: ProofSubstate =
                    system_api.node_drop(RENodeId::Proof(input.proof.0))?.into();
                proof.change_to_unrestricted();

                let mut substate_mut = system_api.get_ref_mut(auth_zone_handle)?;
                let mut raw_mut = substate_mut.get_raw_mut();
                let auth_zone = raw_mut.auth_zone();
                auth_zone.cur_auth_zone_mut().push(proof);
                substate_mut.flush()?;

                ScryptoValue::from_typed(&())
            }
            AuthZoneMethod::CreateProof => {
                let input: AuthZoneCreateProofInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(AuthZoneError::InvalidRequestData(e)))?;

                let resource_type = {
                    let resource_id =
                        RENodeId::Global(GlobalAddress::Resource(input.resource_address));
                    let offset =
                        SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                    let resource_handle =
                        system_api.lock_substate(resource_id, offset, LockFlags::empty())?;
                    let substate_ref = system_api.get_ref(resource_handle)?;
                    substate_ref.resource_manager().resource_type
                };

                let proof = {
                    let mut substate_mut = system_api.get_ref_mut(auth_zone_handle)?;
                    let mut raw_mut = substate_mut.get_raw_mut();
                    let auth_zone = raw_mut.auth_zone();
                    let proof = auth_zone
                        .cur_auth_zone()
                        .create_proof(input.resource_address, resource_type)?;
                    substate_mut.flush()?;
                    proof
                };

                let proof_id = system_api.node_create(HeapRENode::Proof(proof))?.into();
                ScryptoValue::from_typed(&scrypto::resource::Proof(proof_id))
            }
            AuthZoneMethod::CreateProofByAmount => {
                let input: AuthZoneCreateProofByAmountInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(AuthZoneError::InvalidRequestData(e)))?;

                let resource_type = {
                    let resource_id =
                        RENodeId::Global(GlobalAddress::Resource(input.resource_address));
                    let offset =
                        SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                    let resource_handle =
                        system_api.lock_substate(resource_id, offset, LockFlags::empty())?;
                    let substate_ref = system_api.get_ref(resource_handle)?;
                    substate_ref.resource_manager().resource_type
                };

                let proof = {
                    let mut substate_mut = system_api.get_ref_mut(auth_zone_handle)?;
                    let mut raw_mut = substate_mut.get_raw_mut();
                    let auth_zone = raw_mut.auth_zone();
                    let proof = auth_zone.cur_auth_zone().create_proof_by_amount(
                        input.amount,
                        input.resource_address,
                        resource_type,
                    )?;
                    substate_mut.flush()?;
                    proof
                };

                let proof_id = system_api.node_create(HeapRENode::Proof(proof))?.into();
                ScryptoValue::from_typed(&scrypto::resource::Proof(proof_id))
            }
            AuthZoneMethod::CreateProofByIds => {
                let input: AuthZoneCreateProofByIdsInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(AuthZoneError::InvalidRequestData(e)))?;

                let resource_type = {
                    let resource_id =
                        RENodeId::Global(GlobalAddress::Resource(input.resource_address));
                    let offset =
                        SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                    let resource_handle =
                        system_api.lock_substate(resource_id, offset, LockFlags::read_only())?;
                    let substate_ref = system_api.get_ref(resource_handle)?;
                    substate_ref.resource_manager().resource_type
                };

                let proof = {
                    let substate_ref = system_api.get_ref(auth_zone_handle)?;
                    let auth_zone = substate_ref.auth_zone();
                    let proof = auth_zone.cur_auth_zone().create_proof_by_ids(
                        &input.ids,
                        input.resource_address,
                        resource_type,
                    )?;
                    proof
                };

                let proof_id = system_api.node_create(HeapRENode::Proof(proof))?.into();
                ScryptoValue::from_typed(&scrypto::resource::Proof(proof_id))
            }
            AuthZoneMethod::Clear => {
                let _: AuthZoneClearInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(AuthZoneError::InvalidRequestData(e)))?;
                let mut substate_mut = system_api.get_ref_mut(auth_zone_handle)?;
                let mut raw_mut = substate_mut.get_raw_mut();
                let auth_zone = raw_mut.auth_zone();
                auth_zone.cur_auth_zone_mut().clear();
                substate_mut.flush()?;
                ScryptoValue::from_typed(&())
            }
            AuthZoneMethod::Drain => {
                let _: AuthZoneDrainInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(AuthZoneError::InvalidRequestData(e)))?;

                let proofs = {
                    let mut substate_mut = system_api.get_ref_mut(auth_zone_handle)?;
                    let mut raw_mut = substate_mut.get_raw_mut();
                    let auth_zone = raw_mut.auth_zone();
                    let proofs = auth_zone.cur_auth_zone_mut().drain();
                    substate_mut.flush()?;
                    proofs
                };

                let mut proof_ids: Vec<scrypto::resource::Proof> = Vec::new();
                for proof in proofs {
                    let proof_id: ProofId =
                        system_api.node_create(HeapRENode::Proof(proof))?.into();
                    proof_ids.push(scrypto::resource::Proof(proof_id));
                }

                ScryptoValue::from_typed(&proof_ids)
            }
        };

        Ok(rtn)
    }
}
