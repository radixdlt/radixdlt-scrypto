use crate::engine::{
    ApplicationError, CallFrameUpdate, LockFlags, NativeExecutable, NativeInvocation,
    NativeInvocationInfo, RENode, RuntimeError, SystemApi,
};
use crate::model::{InvokeError, ResourceOperationError};
use crate::types::*;
use radix_engine_lib::engine::types::{
    GlobalAddress, NativeMethod, ProofMethod, ProofOffset, RENodeId, SubstateOffset,
};
use radix_engine_lib::model::*;

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

impl NativeExecutable for ProofGetAmountInvocation {
    type NativeOutput = Decimal;

    fn execute<Y>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(Decimal, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Proof(input.receiver);
        let offset = SubstateOffset::Proof(ProofOffset::Proof);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;
        let substate_ref = system_api.get_ref(handle)?;
        let proof = substate_ref.proof();

        Ok((proof.total_amount(), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for ProofGetAmountInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Proof(ProofMethod::GetAmount),
            RENodeId::Proof(self.receiver),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for ProofGetNonFungibleIdsInvocation {
    type NativeOutput = BTreeSet<NonFungibleId>;

    fn execute<Y>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(BTreeSet<NonFungibleId>, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Proof(input.receiver);
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

impl NativeInvocation for ProofGetNonFungibleIdsInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Proof(ProofMethod::GetNonFungibleIds),
            RENodeId::Proof(self.receiver),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for ProofGetResourceAddressInvocation {
    type NativeOutput = ResourceAddress;

    fn execute<Y>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(ResourceAddress, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Proof(input.receiver);
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

impl NativeInvocation for ProofGetResourceAddressInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Proof(ProofMethod::GetResourceAddress),
            RENodeId::Proof(self.receiver),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for ProofCloneInvocation {
    type NativeOutput = Proof;

    fn execute<Y>(input: Self, system_api: &mut Y) -> Result<(Proof, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Proof(input.receiver);
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

impl NativeInvocation for ProofCloneInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Proof(ProofMethod::Clone),
            RENodeId::Proof(self.receiver),
            CallFrameUpdate::empty(),
        )
    }
}
