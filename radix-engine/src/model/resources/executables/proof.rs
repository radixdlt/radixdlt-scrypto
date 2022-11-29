use crate::engine::{
    ApplicationError, CallFrameUpdate, ExecutableInvocation, LockFlags, MethodDeref,
    NativeExecutor, NativeProcedure, REActor, RENode, ResolvedMethod, ResolvedReceiver,
    RuntimeError, SystemApi,
};
use crate::model::{InvokeError, ResourceOperationError};
use crate::types::*;
use radix_engine_interface::api::types::{
    GlobalAddress, NativeMethod, ProofMethod, ProofOffset, RENodeId, SubstateOffset,
};
use radix_engine_interface::data::IndexedScryptoValue;
use radix_engine_interface::model::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
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
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let input = IndexedScryptoValue::from_typed(&self);
        let receiver = RENodeId::Proof(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Proof(ProofMethod::GetAmount)),
            ResolvedReceiver::new(receiver),
        );
        let executor = NativeExecutor(self, input);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for ProofGetAmountInvocation {
    type Output = Decimal;

    fn main<Y>(self, system_api: &mut Y) -> Result<(Decimal, CallFrameUpdate), RuntimeError>
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

impl ExecutableInvocation for ProofGetNonFungibleIdsInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let input = IndexedScryptoValue::from_typed(&self);
        let receiver = RENodeId::Proof(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Proof(ProofMethod::GetNonFungibleIds)),
            ResolvedReceiver::new(receiver),
        );
        let executor = NativeExecutor(self, input);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for ProofGetNonFungibleIdsInvocation {
    type Output = BTreeSet<NonFungibleId>;

    fn main<Y>(
        self,
        system_api: &mut Y,
    ) -> Result<(BTreeSet<NonFungibleId>, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Proof(self.receiver);
        let offset = SubstateOffset::Proof(ProofOffset::Proof);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;
        let substate_ref = system_api.get_ref(handle)?;
        let proof = substate_ref.proof();
        let ids = proof.total_ids().map_err(|e| match e {
            InvokeError::Error(e) => {
                RuntimeError::ApplicationError(ApplicationError::ProofError(e))
            }
            InvokeError::Downstream(runtime_error) => runtime_error,
        })?;

        Ok((ids, CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ProofGetResourceAddressInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let input = IndexedScryptoValue::from_typed(&self);
        let receiver = RENodeId::Proof(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Proof(ProofMethod::GetResourceAddress)),
            ResolvedReceiver::new(receiver),
        );
        let executor = NativeExecutor(self, input);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for ProofGetResourceAddressInvocation {
    type Output = ResourceAddress;

    fn main<Y>(self, system_api: &mut Y) -> Result<(ResourceAddress, CallFrameUpdate), RuntimeError>
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
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let input = IndexedScryptoValue::from_typed(&self);
        let receiver = RENodeId::Proof(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Proof(ProofMethod::Clone)),
            ResolvedReceiver::new(receiver),
        );
        let executor = NativeExecutor(self, input);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for ProofCloneInvocation {
    type Output = Proof;

    fn main<Y>(self, system_api: &mut Y) -> Result<(Proof, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Proof(self.receiver);
        let offset = SubstateOffset::Proof(ProofOffset::Proof);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;
        let substate_ref = system_api.get_ref(handle)?;
        let proof = substate_ref.proof();
        let cloned_proof = proof.clone();
        let proof_id = system_api.create_node(RENode::Proof(cloned_proof))?.into();

        Ok((
            Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}
