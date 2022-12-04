use crate::engine::{
    ApplicationError, CallFrameUpdate, ExecutableInvocation, LockFlags, ResolveApi,
    NativeExecutor, NativeProcedure, REActor, RENode, ResolvedMethod, ResolvedReceiver,
    RuntimeError, SystemApi,
};
use crate::model::{
    convert, InvokeError, MethodAuthorization, MethodAuthorizationError, ProofError,
};
use crate::types::*;
use radix_engine_interface::api::types::{
    AuthZoneStackMethod, AuthZoneStackOffset, GlobalAddress, NativeMethod, ProofOffset, RENodeId,
    ResourceManagerOffset, SubstateOffset,
};
use radix_engine_interface::data::IndexedScryptoValue;
use radix_engine_interface::model::*;
use sbor::rust::vec::Vec;
use crate::wasm::WasmEngine;

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
    AssertAccessRuleError(MethodAuthorization, MethodAuthorizationError),
}

impl<W:WasmEngine> ExecutableInvocation<W> for AuthZonePopInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: ResolveApi<W>> (
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::AuthZoneStack(self.receiver);
        let resolved_receiver = ResolvedReceiver::new(receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);

        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::AuthZoneStack(AuthZoneStackMethod::Pop)),
            resolved_receiver,
        );

        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for AuthZonePopInvocation {
    type Output = Proof;

    fn main<Y>(self, api: &mut Y) -> Result<(Proof, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::AuthZoneStack(self.receiver);
        let offset = SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack);
        let auth_zone_handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let proof = {
            let mut substate_mut = api.get_ref_mut(auth_zone_handle)?;
            let auth_zone_stack = substate_mut.auth_zone_stack();
            let proof = auth_zone_stack
                .cur_auth_zone_mut()
                .pop()
                .map_err(|e| match e {
                    InvokeError::Downstream(runtime_error) => runtime_error,
                    InvokeError::Error(e) => {
                        RuntimeError::ApplicationError(ApplicationError::AuthZoneError(e))
                    }
                })?;
            proof
        };

        let node_id = api.allocate_node_id(RENodeType::Proof)?;
        api.create_node(node_id, RENode::Proof(proof))?;
        let proof_id = node_id.into();

        Ok((
            Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}

impl<W:WasmEngine> ExecutableInvocation<W> for AuthZonePushInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: ResolveApi<W>> (
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::AuthZoneStack(self.receiver);
        let resolved_receiver = ResolvedReceiver::new(receiver);
        let mut call_frame_update = CallFrameUpdate::copy_ref(receiver);
        call_frame_update
            .nodes_to_move
            .push(RENodeId::Proof(self.proof.0));

        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::AuthZoneStack(AuthZoneStackMethod::Push)),
            resolved_receiver,
        );

        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for AuthZonePushInvocation {
    type Output = ();

    fn main<Y>(self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::AuthZoneStack(self.receiver);
        let offset = SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack);
        let auth_zone_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let node_id = RENodeId::Proof(self.proof.0);
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
        let auth_zone_stack = substate_mut.auth_zone_stack();
        auth_zone_stack.cur_auth_zone_mut().push(cloned_proof);

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl<W:WasmEngine> ExecutableInvocation<W> for AuthZoneCreateProofInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: ResolveApi<W>> (
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::AuthZoneStack(self.receiver);
        let resolved_receiver = ResolvedReceiver::new(receiver);
        let mut call_frame_update = CallFrameUpdate::copy_ref(receiver);
        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::Global(GlobalAddress::Resource(
                self.resource_address,
            )));

        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::AuthZoneStack(
                AuthZoneStackMethod::CreateProof,
            )),
            resolved_receiver,
        );

        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for AuthZoneCreateProofInvocation {
    type Output = Proof;

    fn main<Y>(self, api: &mut Y) -> Result<(Proof, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::AuthZoneStack(self.receiver);
        let offset = SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack);
        let auth_zone_handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let resource_type = {
            let resource_id = RENodeId::Global(GlobalAddress::Resource(self.resource_address));
            let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
            let resource_handle = api.lock_substate(resource_id, offset, LockFlags::read_only())?;
            let substate_ref = api.get_ref(resource_handle)?;
            substate_ref.resource_manager().resource_type
        };

        let proof = {
            let mut substate_mut = api.get_ref_mut(auth_zone_handle)?;
            let auth_zone_stack = substate_mut.auth_zone_stack();
            let proof = auth_zone_stack
                .cur_auth_zone()
                .create_proof(self.resource_address, resource_type)
                .map_err(|e| match e {
                    InvokeError::Downstream(runtime_error) => runtime_error,
                    InvokeError::Error(e) => {
                        RuntimeError::ApplicationError(ApplicationError::AuthZoneError(e))
                    }
                })?;
            proof
        };

        let node_id = api.allocate_node_id(RENodeType::Proof)?;
        api.create_node(node_id, RENode::Proof(proof))?;
        let proof_id = node_id.into();

        Ok((
            Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}

impl<W:WasmEngine> ExecutableInvocation<W> for AuthZoneCreateProofByAmountInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: ResolveApi<W>> (
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::AuthZoneStack(self.receiver);
        let resolved_receiver = ResolvedReceiver::new(receiver);
        let mut call_frame_update = CallFrameUpdate::copy_ref(receiver);
        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::Global(GlobalAddress::Resource(
                self.resource_address,
            )));

        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::AuthZoneStack(
                AuthZoneStackMethod::CreateProofByAmount,
            )),
            resolved_receiver,
        );

        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for AuthZoneCreateProofByAmountInvocation {
    type Output = Proof;

    fn main<Y>(self, api: &mut Y) -> Result<(Proof, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::AuthZoneStack(self.receiver);
        let offset = SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack);
        let auth_zone_handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let resource_type = {
            let resource_id = RENodeId::Global(GlobalAddress::Resource(self.resource_address));
            let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
            let resource_handle = api.lock_substate(resource_id, offset, LockFlags::read_only())?;
            let substate_ref = api.get_ref(resource_handle)?;
            substate_ref.resource_manager().resource_type
        };

        let proof = {
            let mut substate_mut = api.get_ref_mut(auth_zone_handle)?;
            let auth_zone_stack = substate_mut.auth_zone_stack();
            let proof = auth_zone_stack
                .cur_auth_zone()
                .create_proof_by_amount(self.amount, self.resource_address, resource_type)
                .map_err(|e| match e {
                    InvokeError::Downstream(runtime_error) => runtime_error,
                    InvokeError::Error(e) => {
                        RuntimeError::ApplicationError(ApplicationError::AuthZoneError(e))
                    }
                })?;

            proof
        };

        let node_id = api.allocate_node_id(RENodeType::Proof)?;
        api.create_node(node_id, RENode::Proof(proof))?;
        let proof_id = node_id.into();

        Ok((
            Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}

impl<W:WasmEngine> ExecutableInvocation<W> for AuthZoneCreateProofByIdsInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: ResolveApi<W>> (
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::AuthZoneStack(self.receiver);
        let resolved_receiver = ResolvedReceiver::new(receiver);
        let mut call_frame_update = CallFrameUpdate::copy_ref(receiver);
        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::Global(GlobalAddress::Resource(
                self.resource_address,
            )));

        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::AuthZoneStack(
                AuthZoneStackMethod::CreateProofByIds,
            )),
            resolved_receiver,
        );

        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for AuthZoneCreateProofByIdsInvocation {
    type Output = Proof;

    fn main<Y>(self, api: &mut Y) -> Result<(Proof, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::AuthZoneStack(self.receiver);
        let offset = SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack);
        let auth_zone_handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let resource_type = {
            let resource_id = RENodeId::Global(GlobalAddress::Resource(self.resource_address));
            let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
            let resource_handle = api.lock_substate(resource_id, offset, LockFlags::read_only())?;
            let substate_ref = api.get_ref(resource_handle)?;
            substate_ref.resource_manager().resource_type
        };

        let proof = {
            let substate_ref = api.get_ref(auth_zone_handle)?;
            let auth_zone_stack = substate_ref.auth_zone_stack();
            let proof = auth_zone_stack
                .cur_auth_zone()
                .create_proof_by_ids(&self.ids, self.resource_address, resource_type)
                .map_err(|e| match e {
                    InvokeError::Downstream(runtime_error) => runtime_error,
                    InvokeError::Error(e) => {
                        RuntimeError::ApplicationError(ApplicationError::AuthZoneError(e))
                    }
                })?;

            proof
        };

        let node_id = api.allocate_node_id(RENodeType::Proof)?;
        api.create_node(node_id, RENode::Proof(proof))?;
        let proof_id = node_id.into();

        Ok((
            Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}

impl<W:WasmEngine> ExecutableInvocation<W> for AuthZoneClearInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: ResolveApi<W>> (
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::AuthZoneStack(self.receiver);
        let resolved_receiver = ResolvedReceiver::new(receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);

        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::AuthZoneStack(AuthZoneStackMethod::Clear)),
            resolved_receiver,
        );

        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for AuthZoneClearInvocation {
    type Output = ();

    fn main<Y>(self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::AuthZoneStack(self.receiver);
        let offset = SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack);
        let auth_zone_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;
        let mut substate_mut = system_api.get_ref_mut(auth_zone_handle)?;
        let auth_zone_stack = substate_mut.auth_zone_stack();
        auth_zone_stack.cur_auth_zone_mut().clear();

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl<W:WasmEngine> ExecutableInvocation<W> for AuthZoneDrainInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: ResolveApi<W>> (
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::AuthZoneStack(self.receiver);
        let resolved_receiver = ResolvedReceiver::new(receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);

        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::AuthZoneStack(AuthZoneStackMethod::Drain)),
            resolved_receiver,
        );

        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for AuthZoneDrainInvocation {
    type Output = Vec<Proof>;

    fn main<Y>(self, api: &mut Y) -> Result<(Vec<Proof>, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::AuthZoneStack(self.receiver);
        let offset = SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack);
        let auth_zone_handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let proofs = {
            let mut substate_mut = api.get_ref_mut(auth_zone_handle)?;
            let auth_zone_stack = substate_mut.auth_zone_stack();
            let proofs = auth_zone_stack.cur_auth_zone_mut().drain();
            proofs
        };

        let mut proof_ids: Vec<Proof> = Vec::new();
        let mut nodes_to_move = Vec::new();
        for proof in proofs {
            let node_id = api.allocate_node_id(RENodeType::Proof)?;
            api.create_node(node_id, RENode::Proof(proof))?;
            let proof_id = node_id.into();
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

impl<W:WasmEngine> ExecutableInvocation<W> for AuthZoneAssertAccessRuleInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: ResolveApi<W>> (
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::AuthZoneStack(self.receiver);
        let resolved_receiver = ResolvedReceiver::new(receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);

        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::AuthZoneStack(
                AuthZoneStackMethod::AssertAccessRule,
            )),
            resolved_receiver,
        );

        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for AuthZoneAssertAccessRuleInvocation {
    type Output = ();

    fn main<Y>(self, api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::AuthZoneStack(self.receiver);
        let offset = SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack);
        let handle = api.lock_substate(node_id, offset, LockFlags::read_only())?;
        let substate_ref = api.get_ref(handle)?;
        let auth_zone_stack = substate_ref.auth_zone_stack();
        let authorization = convert(&Type::Any, &IndexedScryptoValue::unit(), &self.access_rule);

        // Authorization check
        auth_zone_stack
            .check_auth(false, vec![authorization])
            .map_err(|(authorization, error)| {
                RuntimeError::ApplicationError(ApplicationError::AuthZoneError(
                    AuthZoneError::AssertAccessRuleError(authorization, error),
                ))
            })?;

        api.drop_lock(handle)?;

        Ok(((), CallFrameUpdate::empty()))
    }
}
