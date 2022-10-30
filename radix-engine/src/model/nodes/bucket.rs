use crate::engine::{
    ApplicationError, CallFrameUpdate, InvokableNative, LockFlags, NativeExecutable,
    NativeInvocation, NativeInvocationInfo, RENode, RuntimeError, SystemApi,
};
use crate::fee::FeeReserve;
use crate::model::{BucketSubstate, InvokeError, ProofError, ResourceOperationError};
use crate::types::*;

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

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(scrypto::resource::Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R> + InvokableNative<'a>,
        R: FeeReserve,
    {
        let node_id = RENodeId::Bucket(input.bucket_id);
        let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
        let bucket_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(bucket_handle)?;
        let bucket = substate_mut.bucket();
        let container = bucket.take(input.amount).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(
                BucketError::ResourceOperationError(e),
            ))
        })?;
        let bucket_id = system_api
            .create_node(RENode::Bucket(BucketSubstate::new(container)))?
            .into();
        Ok((
            scrypto::resource::Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

impl NativeInvocation for BucketTakeInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Bucket(BucketMethod::Take),
            RENodeId::Bucket(self.bucket_id),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for BucketCreateProofInput {
    type Output = scrypto::resource::Proof;

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(scrypto::resource::Proof, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R> + InvokableNative<'a>,
        R: FeeReserve,
    {
        let node_id = RENodeId::Bucket(input.bucket_id);
        let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
        let bucket_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(bucket_handle)?;
        let bucket = substate_mut.bucket();
        let proof = bucket.create_proof(input.bucket_id).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(BucketError::ProofError(
                e,
            )))
        })?;

        let proof_id = system_api.create_node(RENode::Proof(proof))?.into();
        Ok((
            scrypto::resource::Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}

impl NativeInvocation for BucketCreateProofInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Bucket(BucketMethod::CreateProof),
            RENodeId::Bucket(self.bucket_id),
            CallFrameUpdate::empty(),
        )
    }
}


impl NativeExecutable for BucketTakeNonFungiblesInput {
    type Output = scrypto::resource::Bucket;

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(scrypto::resource::Bucket, CallFrameUpdate), RuntimeError>
        where
            Y: SystemApi<'s, R> + InvokableNative<'a>,
            R: FeeReserve,
    {
        let node_id = RENodeId::Bucket(input.bucket_id);
        let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
        let bucket_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(bucket_handle)?;
        let bucket = substate_mut.bucket();
        let container = bucket
            .take_non_fungibles(&input.ids)
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

impl NativeInvocation for BucketTakeNonFungiblesInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Bucket(BucketMethod::TakeNonFungibles),
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
                panic!("Unexpected")
            }
            BucketMethod::CreateProof => {
                panic!("Unexpected")
            }
            BucketMethod::TakeNonFungibles => {
                panic!("Unexpected")
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
        }?;

        Ok(rtn)
    }
}
