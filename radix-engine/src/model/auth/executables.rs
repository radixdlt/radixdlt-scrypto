use crate::engine::{
    ApplicationError, CallFrameUpdate, LockFlags, NativeInvocation, NativeInvocationInfo, RENode,
    RuntimeError, SystemApi,
};
use crate::model::{InvokeError, ProofError};
use crate::types::*;
use radix_engine_interface::api::types::{
    AuthZoneMethod, AuthZoneOffset, GlobalAddress, NativeMethod, ProofId, ProofOffset, RENodeId,
    ResourceManagerOffset, SubstateOffset,
};
use radix_engine_interface::model::*;
use sbor::rust::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub enum AuthZoneError {
    EmptyAuthZone,
    ProofError(ProofError),
    CouldNotCreateProof,
    InvalidRequestData(DecodeError),
    CouldNotGetProof,
    CouldNotGetResource,
    NoMethodSpecified,
}

impl NativeInvocation for AuthZonePopInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::AuthZone(AuthZoneMethod::Pop),
            RENodeId::AuthZoneStack(self.receiver),
            CallFrameUpdate::empty(),
        )
    }

    fn execute<Y>(input: Self, system_api: &mut Y) -> Result<(Proof, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::AuthZoneStack(input.receiver);
        let offset = SubstateOffset::AuthZone(AuthZoneOffset::AuthZone);
        let auth_zone_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let proof = {
            let mut substate_mut = system_api.get_ref_mut(auth_zone_handle)?;
            let auth_zone = substate_mut.auth_zone();
            let proof = auth_zone.cur_auth_zone_mut().pop().map_err(|e| match e {
                InvokeError::Downstream(runtime_error) => runtime_error,
                InvokeError::Error(e) => {
                    RuntimeError::ApplicationError(ApplicationError::AuthZoneError(e))
                }
            })?;
            proof
        };

        let proof_id = system_api.create_node(RENode::Proof(proof))?.into();

        Ok((
            Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}

impl NativeInvocation for AuthZonePushInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::AuthZone(AuthZoneMethod::Push),
            RENodeId::AuthZoneStack(self.receiver),
            CallFrameUpdate::move_node(RENodeId::Proof(self.proof.0)),
        )
    }

    fn execute<Y>(input: Self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::AuthZoneStack(input.receiver);
        let offset = SubstateOffset::AuthZone(AuthZoneOffset::AuthZone);
        let auth_zone_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let node_id = RENodeId::Proof(input.proof.0);
        let handle = system_api.lock_substate(
            node_id,
            SubstateOffset::Proof(ProofOffset::Proof),
            LockFlags::read_only(),
        )?;
        let substate_ref = system_api.get_ref(handle)?;
        let proof = substate_ref.proof();
        // Take control of the proof lock as the proof in the call frame will lose it's lock once dropped
        let mut cloned_proof = proof.clone();
        cloned_proof.change_to_unrestricted();

        let mut substate_mut = system_api.get_ref_mut(auth_zone_handle)?;
        let auth_zone = substate_mut.auth_zone();
        auth_zone.cur_auth_zone_mut().push(cloned_proof);

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for AuthZoneCreateProofInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::AuthZone(AuthZoneMethod::CreateProof),
            RENodeId::AuthZoneStack(self.receiver),
            CallFrameUpdate::copy_ref(RENodeId::Global(GlobalAddress::Resource(
                self.resource_address,
            ))),
        )
    }

    fn execute<Y>(input: Self, system_api: &mut Y) -> Result<(Proof, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::AuthZoneStack(input.receiver);
        let offset = SubstateOffset::AuthZone(AuthZoneOffset::AuthZone);
        let auth_zone_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let resource_type = {
            let resource_id = RENodeId::Global(GlobalAddress::Resource(input.resource_address));
            let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
            let resource_handle =
                system_api.lock_substate(resource_id, offset, LockFlags::read_only())?;
            let substate_ref = system_api.get_ref(resource_handle)?;
            substate_ref.resource_manager().resource_type
        };

        let proof = {
            let mut substate_mut = system_api.get_ref_mut(auth_zone_handle)?;
            let auth_zone = substate_mut.auth_zone();
            let proof = auth_zone
                .cur_auth_zone()
                .create_proof(input.resource_address, resource_type)
                .map_err(|e| match e {
                    InvokeError::Downstream(runtime_error) => runtime_error,
                    InvokeError::Error(e) => {
                        RuntimeError::ApplicationError(ApplicationError::AuthZoneError(e))
                    }
                })?;
            proof
        };

        let proof_id = system_api.create_node(RENode::Proof(proof))?.into();

        Ok((
            Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}

impl NativeInvocation for AuthZoneCreateProofByAmountInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::AuthZone(AuthZoneMethod::CreateProofByAmount),
            RENodeId::AuthZoneStack(self.receiver),
            CallFrameUpdate::copy_ref(RENodeId::Global(GlobalAddress::Resource(
                self.resource_address,
            ))),
        )
    }

    fn execute<Y>(input: Self, system_api: &mut Y) -> Result<(Proof, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::AuthZoneStack(input.receiver);
        let offset = SubstateOffset::AuthZone(AuthZoneOffset::AuthZone);
        let auth_zone_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let resource_type = {
            let resource_id = RENodeId::Global(GlobalAddress::Resource(input.resource_address));
            let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
            let resource_handle =
                system_api.lock_substate(resource_id, offset, LockFlags::read_only())?;
            let substate_ref = system_api.get_ref(resource_handle)?;
            substate_ref.resource_manager().resource_type
        };

        let proof = {
            let mut substate_mut = system_api.get_ref_mut(auth_zone_handle)?;
            let auth_zone = substate_mut.auth_zone();
            let proof = auth_zone
                .cur_auth_zone()
                .create_proof_by_amount(input.amount, input.resource_address, resource_type)
                .map_err(|e| match e {
                    InvokeError::Downstream(runtime_error) => runtime_error,
                    InvokeError::Error(e) => {
                        RuntimeError::ApplicationError(ApplicationError::AuthZoneError(e))
                    }
                })?;

            proof
        };

        let proof_id = system_api.create_node(RENode::Proof(proof))?.into();

        Ok((
            Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}

impl NativeInvocation for AuthZoneCreateProofByIdsInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::AuthZone(AuthZoneMethod::CreateProofByIds),
            RENodeId::AuthZoneStack(self.receiver),
            CallFrameUpdate::copy_ref(RENodeId::Global(GlobalAddress::Resource(
                self.resource_address,
            ))),
        )
    }

    fn execute<Y>(input: Self, system_api: &mut Y) -> Result<(Proof, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::AuthZoneStack(input.receiver);
        let offset = SubstateOffset::AuthZone(AuthZoneOffset::AuthZone);
        let auth_zone_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let resource_type = {
            let resource_id = RENodeId::Global(GlobalAddress::Resource(input.resource_address));
            let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
            let resource_handle =
                system_api.lock_substate(resource_id, offset, LockFlags::read_only())?;
            let substate_ref = system_api.get_ref(resource_handle)?;
            substate_ref.resource_manager().resource_type
        };

        let proof = {
            let substate_ref = system_api.get_ref(auth_zone_handle)?;
            let auth_zone = substate_ref.auth_zone();
            let proof = auth_zone
                .cur_auth_zone()
                .create_proof_by_ids(&input.ids, input.resource_address, resource_type)
                .map_err(|e| match e {
                    InvokeError::Downstream(runtime_error) => runtime_error,
                    InvokeError::Error(e) => {
                        RuntimeError::ApplicationError(ApplicationError::AuthZoneError(e))
                    }
                })?;

            proof
        };

        let proof_id = system_api.create_node(RENode::Proof(proof))?.into();

        Ok((
            Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}

impl NativeInvocation for AuthZoneClearInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::AuthZone(AuthZoneMethod::Clear),
            RENodeId::AuthZoneStack(self.receiver),
            CallFrameUpdate::empty(),
        )
    }

    fn execute<Y>(input: Self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::AuthZoneStack(input.receiver);
        let offset = SubstateOffset::AuthZone(AuthZoneOffset::AuthZone);
        let auth_zone_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;
        let mut substate_mut = system_api.get_ref_mut(auth_zone_handle)?;
        let auth_zone = substate_mut.auth_zone();
        auth_zone.cur_auth_zone_mut().clear();

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for AuthZoneDrainInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::AuthZone(AuthZoneMethod::Drain),
            RENodeId::AuthZoneStack(self.receiver),
            CallFrameUpdate::empty(),
        )
    }

    fn execute<Y>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(Vec<Proof>, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::AuthZoneStack(input.receiver);
        let offset = SubstateOffset::AuthZone(AuthZoneOffset::AuthZone);
        let auth_zone_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let proofs = {
            let mut substate_mut = system_api.get_ref_mut(auth_zone_handle)?;
            let auth_zone = substate_mut.auth_zone();
            let proofs = auth_zone.cur_auth_zone_mut().drain();
            proofs
        };

        let mut proof_ids: Vec<Proof> = Vec::new();
        let mut nodes_to_move = Vec::new();
        for proof in proofs {
            let proof_id: ProofId = system_api.create_node(RENode::Proof(proof))?.into();
            proof_ids.push(Proof(proof_id));
            nodes_to_move.push(RENodeId::Proof(proof_id));
        }

        Ok((
            proof_ids,
            CallFrameUpdate {
                nodes_to_move,
                node_refs_to_copy: HashSet::new(),
            },
        ))
    }
}
