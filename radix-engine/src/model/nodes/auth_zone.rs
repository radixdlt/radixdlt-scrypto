use crate::engine::{
    ApplicationError, CallFrameUpdate, InvokableNative, LockFlags, NativeExecutable,
    NativeInvocation, NativeInvocationInfo, RENode, RuntimeError, SystemApi,
};
use crate::fee::FeeReserve;
use crate::model::{InvokeError, ProofError};
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

impl NativeExecutable for AuthZonePopInput {
    type Output = scrypto::resource::Proof;

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(scrypto::resource::Proof, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R> + InvokableNative<'a>,
        R: FeeReserve,
    {
        let node_id = RENodeId::AuthZoneStack(input.auth_zone_id);
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
            scrypto::resource::Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}

impl NativeInvocation for AuthZonePopInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::AuthZone(AuthZoneMethod::Pop),
            RENodeId::AuthZoneStack(self.auth_zone_id),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for AuthZonePushInput {
    type Output = ();

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R> + InvokableNative<'a>,
        R: FeeReserve,
    {
        let node_id = RENodeId::AuthZoneStack(input.auth_zone_id);
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

impl NativeInvocation for AuthZonePushInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::AuthZone(AuthZoneMethod::Push),
            RENodeId::AuthZoneStack(self.auth_zone_id),
            CallFrameUpdate::move_node(RENodeId::Proof(self.proof.0)),
        )
    }
}

impl NativeExecutable for AuthZoneCreateProofInput {
    type Output = scrypto::resource::Proof;

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(scrypto::resource::Proof, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R> + InvokableNative<'a>,
        R: FeeReserve,
    {
        let node_id = RENodeId::AuthZoneStack(input.auth_zone_id);
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
            scrypto::resource::Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}

impl NativeInvocation for AuthZoneCreateProofInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::AuthZone(AuthZoneMethod::CreateProof),
            RENodeId::AuthZoneStack(self.auth_zone_id),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for AuthZoneCreateProofByAmountInput {
    type Output = scrypto::resource::Proof;

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(scrypto::resource::Proof, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R> + InvokableNative<'a>,
        R: FeeReserve,
    {
        let node_id = RENodeId::AuthZoneStack(input.auth_zone_id);
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
            scrypto::resource::Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}

impl NativeInvocation for AuthZoneCreateProofByAmountInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::AuthZone(AuthZoneMethod::CreateProofByAmount),
            RENodeId::AuthZoneStack(self.auth_zone_id),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for AuthZoneCreateProofByIdsInput {
    type Output = scrypto::resource::Proof;

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(scrypto::resource::Proof, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R> + InvokableNative<'a>,
        R: FeeReserve,
    {
        let node_id = RENodeId::AuthZoneStack(input.auth_zone_id);
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
            scrypto::resource::Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}

impl NativeInvocation for AuthZoneCreateProofByIdsInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::AuthZone(AuthZoneMethod::CreateProofByIds),
            RENodeId::AuthZoneStack(self.auth_zone_id),
            CallFrameUpdate::empty(),
        )
    }
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
                panic!("Unexpected")
            }
            AuthZoneMethod::Push => {
                panic!("Unexpected")
            }
            AuthZoneMethod::CreateProof => {
                panic!("Unexpected")
            }
            AuthZoneMethod::CreateProofByAmount => {
                panic!("Unexpected")
            }
            AuthZoneMethod::CreateProofByIds => {
                panic!("Unexpected")
            }
            AuthZoneMethod::Clear => {
                let _: AuthZoneClearInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(AuthZoneError::InvalidRequestData(e)))?;
                let mut substate_mut = system_api.get_ref_mut(auth_zone_handle)?;
                let auth_zone = substate_mut.auth_zone();
                auth_zone.cur_auth_zone_mut().clear();
                ScryptoValue::from_typed(&())
            }
            AuthZoneMethod::Drain => {
                let _: AuthZoneDrainInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(AuthZoneError::InvalidRequestData(e)))?;

                let proofs = {
                    let mut substate_mut = system_api.get_ref_mut(auth_zone_handle)?;
                    let auth_zone = substate_mut.auth_zone();
                    let proofs = auth_zone.cur_auth_zone_mut().drain();
                    proofs
                };

                let mut proof_ids: Vec<scrypto::resource::Proof> = Vec::new();
                for proof in proofs {
                    let proof_id: ProofId = system_api.create_node(RENode::Proof(proof))?.into();
                    proof_ids.push(scrypto::resource::Proof(proof_id));
                }

                ScryptoValue::from_typed(&proof_ids)
            }
        };

        Ok(rtn)
    }
}
