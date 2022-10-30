use sbor::Encoder;
use crate::engine::{ApplicationError, CallFrameUpdate, Invocation, Invokable, InvokableNativeFunction, LockFlags, NativeExecutable, NativeMethInvocation, RENode, RuntimeError, SystemApi};
use crate::fee::FeeReserve;
use crate::model::{BucketSubstate, InvokeError, ProofError, ResourceOperationError};
use crate::types::*;
use scrypto::resource::Proof;

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum BucketError {
    InvalidDivisibility,
    InvalidRequestData(DecodeError),
    CouldNotCreateBucket,
    CouldNotTakeBucket,
    ResourceOperationError(ResourceOperationError),
    ProofError(ProofError),
    CouldNotCreateProof,
    MethodNotFound(BucketMethod),
}

impl NativeExecutable for BucketTakeInput {
    type Output = scrypto::resource::Bucket;

    fn execute<'s, 'a, Y, R>(input: Self, system_api: &mut Y) -> Result<(scrypto::resource::Bucket, CallFrameUpdate), RuntimeError> where Y: SystemApi<'s, R> + Invokable<ScryptoInvocation> + InvokableNativeFunction<'a> + Invokable<NativeMethodInvocation>, R: FeeReserve {
        let node_id = RENodeId::Bucket(input.bucket_id);
        let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
        let bucket_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(bucket_handle)?;
        let bucket = substate_mut.bucket();
        let container = bucket
            .take(input.amount)
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::BucketError(BucketError::ResourceOperationError(e))))?;
        let bucket_id = system_api
            .create_node(RENode::Bucket(BucketSubstate::new(container)))?
            .into();
        Ok((
            scrypto::resource::Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

impl NativeMethInvocation for BucketTakeInput {
    fn native_method() -> NativeMethod {
        NativeMethod::Bucket(BucketMethod::Take)
    }

    fn call_frame_update(&self) -> (RENodeId, CallFrameUpdate) {
        (
            RENodeId::Bucket(self.bucket_id),
            CallFrameUpdate::empty(),
        )
    }
}

pub struct Bucket;

trait BucketMethodActor<I, O, E> {
    fn execute<'s, Y, R>(
        bucket_id: BucketId,
        input: I,
        system_api: &mut Y,
    ) -> Result<O, InvokeError<E>>
    where
        Y: SystemApi<'s, R>,
        R: FeeReserve;
}

impl BucketMethodActor<BucketTakeInput, scrypto::resource::Bucket, BucketError> for Bucket {
    fn execute<'s, Y, R>(
        bucket_id: BucketId,
        input: BucketTakeInput,
        system_api: &mut Y,
    ) -> Result<scrypto::resource::Bucket, InvokeError<BucketError>>
    where
        Y: SystemApi<'s, R>,
        R: FeeReserve,
    {
        let node_id = RENodeId::Bucket(bucket_id);
        let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
        let bucket_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(bucket_handle)?;
        let bucket = substate_mut.bucket();
        let container = bucket
            .take(input.amount)
            .map_err(|e| InvokeError::Error(BucketError::ResourceOperationError(e)))?;
        let bucket_id = system_api
            .create_node(RENode::Bucket(BucketSubstate::new(container)))?
            .into();
        Ok(scrypto::resource::Bucket(bucket_id))
    }
}

impl BucketMethodActor<BucketTakeNonFungiblesInput, scrypto::resource::Bucket, BucketError>
    for Bucket
{
    fn execute<'s, Y, R>(
        bucket_id: BucketId,
        input: BucketTakeNonFungiblesInput,
        system_api: &mut Y,
    ) -> Result<scrypto::resource::Bucket, InvokeError<BucketError>>
    where
        Y: SystemApi<'s, R>,
        R: FeeReserve,
    {
        let node_id = RENodeId::Bucket(bucket_id);
        let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
        let bucket_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(bucket_handle)?;
        let bucket = substate_mut.bucket();
        let container = bucket
            .take_non_fungibles(&input.ids)
            .map_err(|e| InvokeError::Error(BucketError::ResourceOperationError(e)))?;
        let bucket_id = system_api
            .create_node(RENode::Bucket(BucketSubstate::new(container)))?
            .into();
        Ok(scrypto::resource::Bucket(bucket_id))
    }
}

impl BucketMethodActor<BucketGetNonFungibleIdsInput, BTreeSet<NonFungibleId>, BucketError>
    for Bucket
{
    fn execute<'s, Y, R>(
        bucket_id: BucketId,
        _input: BucketGetNonFungibleIdsInput,
        system_api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleId>, InvokeError<BucketError>>
    where
        Y: SystemApi<'s, R>,
        R: FeeReserve,
    {
        let node_id = RENodeId::Bucket(bucket_id);
        let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
        let bucket_handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate_ref = system_api.get_ref(bucket_handle)?;
        let bucket = substate_ref.bucket();
        bucket
            .total_ids()
            .map_err(|e| InvokeError::Error(BucketError::ResourceOperationError(e)))
    }
}

impl BucketMethodActor<BucketGetAmountInput, Decimal, BucketError> for Bucket {
    fn execute<'s, Y, R>(
        bucket_id: BucketId,
        _input: BucketGetAmountInput,
        system_api: &mut Y,
    ) -> Result<Decimal, InvokeError<BucketError>>
    where
        Y: SystemApi<'s, R>,
        R: FeeReserve,
    {
        let node_id = RENodeId::Bucket(bucket_id);
        let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
        let bucket_handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate = system_api.get_ref(bucket_handle)?;
        let bucket = substate.bucket();
        Ok(bucket.total_amount())
    }
}

impl BucketMethodActor<BucketPutInput, (), BucketError> for Bucket {
    fn execute<'s, Y, R>(
        bucket_id: BucketId,
        input: BucketPutInput,
        system_api: &mut Y,
    ) -> Result<(), InvokeError<BucketError>>
    where
        Y: SystemApi<'s, R>,
        R: FeeReserve,
    {
        let node_id = RENodeId::Bucket(bucket_id);
        let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
        let bucket_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let other_bucket = system_api
            .drop_node(RENodeId::Bucket(input.bucket.0))?
            .into();
        let mut substate_mut = system_api.get_ref_mut(bucket_handle)?;
        let bucket = substate_mut.bucket();
        bucket
            .put(other_bucket)
            .map_err(|e| InvokeError::Error(BucketError::ResourceOperationError(e)))?;
        Ok(())
    }
}

impl BucketMethodActor<BucketGetResourceAddressInput, ResourceAddress, BucketError> for Bucket {
    fn execute<'s, Y, R>(
        bucket_id: BucketId,
        _input: BucketGetResourceAddressInput,
        system_api: &mut Y,
    ) -> Result<ResourceAddress, InvokeError<BucketError>>
    where
        Y: SystemApi<'s, R>,
        R: FeeReserve,
    {
        let node_id = RENodeId::Bucket(bucket_id);
        let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
        let bucket_handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate = system_api.get_ref(bucket_handle)?;
        let bucket = substate.bucket();
        Ok(bucket.resource_address())
    }
}

impl BucketMethodActor<BucketCreateProofInput, scrypto::resource::Proof, BucketError> for Bucket {
    fn execute<'s, Y, R>(
        bucket_id: BucketId,
        _input: BucketCreateProofInput,
        system_api: &mut Y,
    ) -> Result<Proof, InvokeError<BucketError>>
    where
        Y: SystemApi<'s, R>,
        R: FeeReserve,
    {
        let node_id = RENodeId::Bucket(bucket_id);
        let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
        let bucket_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(bucket_handle)?;
        let bucket = substate_mut.bucket();
        let proof = bucket
            .create_proof(bucket_id)
            .map_err(|e| InvokeError::Error(BucketError::ProofError(e)))?;

        let proof_id = system_api.create_node(RENode::Proof(proof))?.into();
        Ok(scrypto::resource::Proof(proof_id))
    }
}

impl Bucket {
    pub fn main<'s, Y, R>(
        bucket_id: BucketId,
        method: BucketMethod,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<BucketError>>
    where
        Y: SystemApi<'s, R>,
        R: FeeReserve,
    {
        let rtn = match method {
            BucketMethod::Take => {
                let input: BucketTakeInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                Self::execute(bucket_id, input, system_api)
                    .map(|rtn| ScryptoValue::from_typed(&rtn))
            }
            BucketMethod::TakeNonFungibles => {
                let input: BucketTakeNonFungiblesInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                Self::execute(bucket_id, input, system_api)
                    .map(|rtn| ScryptoValue::from_typed(&rtn))
            }
            BucketMethod::GetNonFungibleIds => {
                let input: BucketGetNonFungibleIdsInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                Self::execute(bucket_id, input, system_api)
                    .map(|rtn| ScryptoValue::from_typed(&rtn))
            }
            BucketMethod::Put => {
                let input: BucketPutInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                Self::execute(bucket_id, input, system_api)
                    .map(|rtn| ScryptoValue::from_typed(&rtn))
            }
            BucketMethod::GetAmount => {
                let input: BucketGetAmountInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                Self::execute(bucket_id, input, system_api)
                    .map(|rtn| ScryptoValue::from_typed(&rtn))
            }
            BucketMethod::GetResourceAddress => {
                let input: BucketGetResourceAddressInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                Self::execute(bucket_id, input, system_api)
                    .map(|rtn| ScryptoValue::from_typed(&rtn))
            }
            BucketMethod::CreateProof => {
                let input: BucketCreateProofInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                Self::execute(bucket_id, input, system_api)
                    .map(|rtn| ScryptoValue::from_typed(&rtn))
            }
        }?;

        Ok(rtn)
    }
}
