use crate::engine::{
    CallFrameUpdate, ExecutableInvocation, Executor, LockFlags, RENodeInit, ResolvedActor,
    ResolvedReceiver, ResolverApi, RuntimeError, SystemApi,
};
use crate::model::ResourceOperationError;
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::types::{
    GlobalAddress, NativeFn, ProofFn, ProofOffset, RENodeId, SubstateOffset,
};
use radix_engine_interface::model::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ProofError {
    /// Error produced by a resource container.
    ResourceOperationError(ResourceOperationError),
    /// Can't generate zero-amount or empty non-fungible set proofs.
    EmptyProofNotAllowed,
    /// The base proofs are not enough to cover the requested amount or non-fungible ids.
    InsufficientBaseProofs,
    /// Can't apply a non-fungible operation on fungible proofs.
    NonFungibleOperationNotAllowed,
    /// Can't apply a fungible operation on non-fungible proofs.
    FungibleOperationNotAllowed,
    CouldNotCreateProof,
    InvalidRequestData(DecodeError),
}

impl ExecutableInvocation for ProofGetAmountInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Proof(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = ResolvedActor::method(
            NativeFn::Proof(ProofFn::GetAmount),
            ResolvedReceiver::new(receiver),
        );
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for ProofGetAmountInvocation {
    type Output = Decimal;

    fn execute<Y, W: WasmEngine>(
        self,
        system_api: &mut Y,
    ) -> Result<(Decimal, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Proof(self.receiver);
        let offset = SubstateOffset::Proof(ProofOffset::Proof);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;
        let substate_ref = system_api.get_ref(handle)?;
        let proof = substate_ref.proof();

        Ok((proof.total_amount(), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ProofGetNonFungibleLocalIdsInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Proof(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = ResolvedActor::method(
            NativeFn::Proof(ProofFn::GetNonFungibleLocalIds),
            ResolvedReceiver::new(receiver),
        );
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for ProofGetNonFungibleLocalIdsInvocation {
    type Output = BTreeSet<NonFungibleLocalId>;

    fn execute<Y, W: WasmEngine>(
        self,
        system_api: &mut Y,
    ) -> Result<(BTreeSet<NonFungibleLocalId>, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Proof(self.receiver);
        let offset = SubstateOffset::Proof(ProofOffset::Proof);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;
        let substate_ref = system_api.get_ref(handle)?;
        let proof = substate_ref.proof();
        let ids = proof.total_ids()?;

        Ok((ids, CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ProofGetResourceAddressInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Proof(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = ResolvedActor::method(
            NativeFn::Proof(ProofFn::GetResourceAddress),
            ResolvedReceiver::new(receiver),
        );
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for ProofGetResourceAddressInvocation {
    type Output = ResourceAddress;

    fn execute<Y, W: WasmEngine>(
        self,
        system_api: &mut Y,
    ) -> Result<(ResourceAddress, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Proof(self.receiver);
        let offset = SubstateOffset::Proof(ProofOffset::Proof);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;
        let substate_ref = system_api.get_ref(handle)?;
        let proof = substate_ref.proof();

        Ok((
            proof.resource_address,
            CallFrameUpdate::copy_ref(RENodeId::Global(GlobalAddress::Resource(
                proof.resource_address,
            ))),
        ))
    }
}

impl ExecutableInvocation for ProofCloneInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Proof(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = ResolvedActor::method(
            NativeFn::Proof(ProofFn::Clone),
            ResolvedReceiver::new(receiver),
        );
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for ProofCloneInvocation {
    type Output = Proof;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Proof, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Proof(self.receiver);
        let offset = SubstateOffset::Proof(ProofOffset::Proof);
        let handle = api.lock_substate(node_id, offset, LockFlags::read_only())?;
        let substate_ref = api.get_ref(handle)?;
        let proof = substate_ref.proof();
        let cloned_proof = proof.clone();

        let node_id = api.allocate_node_id(RENodeType::Proof)?;
        api.create_node(node_id, RENodeInit::Proof(cloned_proof))?;
        let proof_id = node_id.into();

        Ok((
            Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}
