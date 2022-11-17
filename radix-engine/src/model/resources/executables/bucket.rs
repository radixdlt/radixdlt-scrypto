use crate::engine::{
    ApplicationError, CallFrameUpdate, LockFlags, NativeExecutable, NativeInvocation,
    NativeInvocationInfo, RENode, RuntimeError, SystemApi,
};
use crate::model::{BucketSubstate, ProofError, ResourceOperationError};
use crate::types::*;
use radix_engine_interface::engine::types::{
    BucketMethod, BucketOffset, GlobalAddress, NativeMethod, RENodeId, SubstateOffset,
};
use radix_engine_interface::model::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
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

impl NativeExecutable for BucketTakeInvocation {
    type NativeOutput = Bucket;

    fn execute<Y>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Bucket(input.receiver);
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
            Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

impl NativeInvocation for BucketTakeInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Bucket(BucketMethod::Take),
            RENodeId::Bucket(self.receiver),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for BucketCreateProofInvocation {
    type NativeOutput = Proof;

    fn execute<Y>(input: Self, system_api: &mut Y) -> Result<(Proof, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Bucket(input.receiver);
        let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
        let bucket_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(bucket_handle)?;
        let bucket = substate_mut.bucket();
        let proof = bucket.create_proof(input.receiver).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(BucketError::ProofError(
                e,
            )))
        })?;

        let proof_id = system_api.create_node(RENode::Proof(proof))?.into();
        Ok((
            Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}

impl NativeInvocation for BucketCreateProofInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Bucket(BucketMethod::CreateProof),
            RENodeId::Bucket(self.receiver),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for BucketTakeNonFungiblesInvocation {
    type NativeOutput = Bucket;

    fn execute<Y>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Bucket(input.receiver);
        let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
        let bucket_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(bucket_handle)?;
        let bucket = substate_mut.bucket();
        let container = bucket.take_non_fungibles(&input.ids).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(
                BucketError::ResourceOperationError(e),
            ))
        })?;
        let bucket_id = system_api
            .create_node(RENode::Bucket(BucketSubstate::new(container)))?
            .into();
        Ok((
            Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

impl NativeInvocation for BucketTakeNonFungiblesInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Bucket(BucketMethod::TakeNonFungibles),
            RENodeId::Bucket(self.receiver),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for BucketGetNonFungibleIdsInvocation {
    type NativeOutput = BTreeSet<NonFungibleId>;

    fn execute<Y>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(BTreeSet<NonFungibleId>, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Bucket(input.receiver);
        let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
        let bucket_handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate_ref = system_api.get_ref(bucket_handle)?;
        let bucket = substate_ref.bucket();
        let ids = bucket.total_ids().map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(
                BucketError::ResourceOperationError(e),
            ))
        })?;

        Ok((ids, CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for BucketGetNonFungibleIdsInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Bucket(BucketMethod::GetNonFungibleIds),
            RENodeId::Bucket(self.receiver),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for BucketGetAmountInvocation {
    type NativeOutput = Decimal;

    fn execute<Y>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(Decimal, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Bucket(input.receiver);
        let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
        let bucket_handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate = system_api.get_ref(bucket_handle)?;
        let bucket = substate.bucket();
        Ok((bucket.total_amount(), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for BucketGetAmountInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Bucket(BucketMethod::GetAmount),
            RENodeId::Bucket(self.receiver),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for BucketPutInvocation {
    type NativeOutput = ();

    fn execute<Y>(input: Self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Bucket(input.receiver);
        let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
        let bucket_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let other_bucket = system_api
            .drop_node(RENodeId::Bucket(input.bucket.0))?
            .into();
        let mut substate_mut = system_api.get_ref_mut(bucket_handle)?;
        let bucket = substate_mut.bucket();
        bucket.put(other_bucket).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(
                BucketError::ResourceOperationError(e),
            ))
        })?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for BucketPutInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Bucket(BucketMethod::Put),
            RENodeId::Bucket(self.receiver),
            CallFrameUpdate::move_node(RENodeId::Bucket(self.bucket.0)),
        )
    }
}

impl NativeExecutable for BucketGetResourceAddressInvocation {
    type NativeOutput = ResourceAddress;

    fn execute<Y>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(ResourceAddress, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Bucket(input.receiver);
        let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
        let bucket_handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate = system_api.get_ref(bucket_handle)?;
        let bucket = substate.bucket();
        Ok((
            bucket.resource_address(),
            CallFrameUpdate::copy_ref(RENodeId::Global(GlobalAddress::Resource(
                bucket.resource_address(),
            ))),
        ))
    }
}

impl NativeInvocation for BucketGetResourceAddressInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Bucket(BucketMethod::GetResourceAddress),
            RENodeId::Bucket(self.receiver),
            CallFrameUpdate::empty(),
        )
    }
}
