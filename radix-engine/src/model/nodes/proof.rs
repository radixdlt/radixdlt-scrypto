use crate::engine::{ApplicationError, CallFrameUpdate, InvokableNative, LockFlags, NativeExecutable, NativeInvocation, NativeInvocationInfo, RENode, RuntimeError, SystemApi};
use crate::fee::FeeReserve;
use crate::model::{InvokeError, ResourceOperationError};
use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
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

impl NativeExecutable for ProofGetAmountInput {
    type Output = Decimal;

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(Decimal, CallFrameUpdate), RuntimeError>
        where
            Y: SystemApi<'s, R> + InvokableNative<'a>,
            R: FeeReserve,
    {
        let node_id = RENodeId::Bucket(input.proof_id);
        let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;
        let substate_ref = system_api.get_ref(handle)?;
        let proof = substate_ref.proof();

        Ok((
            proof.total_amount(),
            CallFrameUpdate::empty(),
        ))
    }
}

impl NativeInvocation for ProofGetAmountInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Proof(ProofMethod::GetAmount),
            RENodeId::Proof(self.proof_id),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for ProofGetNonFungibleIdsInput {
    type Output = BTreeSet<NonFungibleId>;

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(BTreeSet<NonFungibleId>, CallFrameUpdate), RuntimeError>
        where
            Y: SystemApi<'s, R> + InvokableNative<'a>,
            R: FeeReserve,
    {
        let node_id = RENodeId::Bucket(input.proof_id);
        let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;
        let substate_ref = system_api.get_ref(handle)?;
        let proof = substate_ref.proof();
        let ids = proof.total_ids().map_err(|e| {
            match e {
                InvokeError::Error(e) => RuntimeError::ApplicationError(ApplicationError::ProofError(e)),
                InvokeError::Downstream(runtime_error) => runtime_error,
            }
        })?;

        Ok((
            ids,
            CallFrameUpdate::empty(),
        ))
    }
}

impl NativeInvocation for ProofGetNonFungibleIdsInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Proof(ProofMethod::GetNonFungibleIds),
            RENodeId::Proof(self.proof_id),
            CallFrameUpdate::empty(),
        )
    }
}


pub struct Proof;

impl Proof {
    pub fn main<'s, Y, R>(
        proof_id: ProofId,
        method: ProofMethod,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<ProofError>>
    where
        Y: SystemApi<'s, R>,
        R: FeeReserve,
    {
        let node_id = RENodeId::Proof(proof_id);
        let offset = SubstateOffset::Proof(ProofOffset::Proof);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;
        let substate_ref = system_api.get_ref(handle)?;
        let proof = substate_ref.proof();

        let rtn = match method {
            ProofMethod::GetAmount => {
                panic!("Unexpected")
            }
            ProofMethod::GetNonFungibleIds => {
                panic!("Unexpected")
            }
            ProofMethod::GetResourceAddress => {
                let _: ProofGetResourceAddressInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ProofError::InvalidRequestData(e)))?;
                ScryptoValue::from_typed(&proof.resource_address)
            }
            ProofMethod::Clone => {
                let _: ProofCloneInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ProofError::InvalidRequestData(e)))?;
                let cloned_proof = proof.clone();
                let proof_id = system_api.create_node(RENode::Proof(cloned_proof))?.into();
                ScryptoValue::from_typed(&scrypto::resource::Proof(proof_id))
            }
        };

        Ok(rtn)
    }
}
