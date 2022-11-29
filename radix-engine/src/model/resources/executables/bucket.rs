use crate::engine::{
    ApplicationError, CallFrameUpdate, ExecutableInvocation, LockFlags, MethodDeref,
    NativeExecutor, NativeProcedure, REActor, RENode, ResolvedMethod, ResolvedReceiver,
    RuntimeError, SystemApi,
};
use crate::model::{BucketSubstate, ProofError, ResourceOperationError};
use crate::types::*;
use radix_engine_interface::api::types::{
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

impl ExecutableInvocation for BucketTakeInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Bucket(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Bucket(BucketMethod::Take)),
            ResolvedReceiver::new(receiver),
        );
        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for BucketTakeInvocation {
    type Output = Bucket;

    fn main<Y>(self, api: &mut Y) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Bucket(self.receiver);
        let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
        let bucket_handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = api.get_ref_mut(bucket_handle)?;
        let bucket = substate_mut.bucket();
        let container = bucket.take(self.amount).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(
                BucketError::ResourceOperationError(e),
            ))
        })?;

        let node_id = api.allocate_node_id(RENodeType::Bucket)?;
        api.create_node(node_id, RENode::Bucket(BucketSubstate::new(container)))?;
        let bucket_id = node_id.into();
        Ok((
            Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

impl ExecutableInvocation for BucketCreateProofInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Bucket(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Bucket(BucketMethod::CreateProof)),
            ResolvedReceiver::new(receiver),
        );
        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for BucketCreateProofInvocation {
    type Output = Proof;

    fn main<Y>(self, api: &mut Y) -> Result<(Proof, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Bucket(self.receiver);
        let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
        let bucket_handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = api.get_ref_mut(bucket_handle)?;
        let bucket = substate_mut.bucket();
        let proof = bucket.create_proof(self.receiver).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(BucketError::ProofError(
                e,
            )))
        })?;

        let node_id = api.allocate_node_id(RENodeType::Proof)?;
        api.create_node(node_id, RENode::Proof(proof))?;
        let proof_id = node_id.into();

        Ok((
            Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}

impl ExecutableInvocation for BucketTakeNonFungiblesInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Bucket(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Bucket(BucketMethod::TakeNonFungibles)),
            ResolvedReceiver::new(receiver),
        );
        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for BucketTakeNonFungiblesInvocation {
    type Output = Bucket;

    fn main<Y>(self, api: &mut Y) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Bucket(self.receiver);
        let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
        let bucket_handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = api.get_ref_mut(bucket_handle)?;
        let bucket = substate_mut.bucket();
        let container = bucket.take_non_fungibles(&self.ids).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::BucketError(
                BucketError::ResourceOperationError(e),
            ))
        })?;

        let node_id = api.allocate_node_id(RENodeType::Proof)?;
        api.create_node(node_id, RENode::Bucket(BucketSubstate::new(container)))?;
        let bucket_id = node_id.into();
        Ok((
            Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

impl ExecutableInvocation for BucketGetNonFungibleIdsInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Bucket(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Bucket(BucketMethod::GetNonFungibleIds)),
            ResolvedReceiver::new(receiver),
        );
        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for BucketGetNonFungibleIdsInvocation {
    type Output = BTreeSet<NonFungibleId>;

    fn main<Y>(
        self,
        system_api: &mut Y,
    ) -> Result<(BTreeSet<NonFungibleId>, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Bucket(self.receiver);
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

impl ExecutableInvocation for BucketGetAmountInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Bucket(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Bucket(BucketMethod::GetAmount)),
            ResolvedReceiver::new(receiver),
        );
        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for BucketGetAmountInvocation {
    type Output = Decimal;

    fn main<Y>(self, system_api: &mut Y) -> Result<(Decimal, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Bucket(self.receiver);
        let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
        let bucket_handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate = system_api.get_ref(bucket_handle)?;
        let bucket = substate.bucket();
        Ok((bucket.total_amount(), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for BucketPutInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Bucket(self.receiver);
        let mut call_frame_update = CallFrameUpdate::copy_ref(receiver);
        call_frame_update
            .nodes_to_move
            .push(RENodeId::Bucket(self.bucket.0));
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Bucket(BucketMethod::Put)),
            ResolvedReceiver::new(receiver),
        );
        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for BucketPutInvocation {
    type Output = ();

    fn main<Y>(self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Bucket(self.receiver);
        let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
        let bucket_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let other_bucket = system_api
            .drop_node(RENodeId::Bucket(self.bucket.0))?
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

impl ExecutableInvocation for BucketGetResourceAddressInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Bucket(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Bucket(BucketMethod::GetResourceAddress)),
            ResolvedReceiver::new(receiver),
        );
        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for BucketGetResourceAddressInvocation {
    type Output = ResourceAddress;

    fn main<Y>(self, system_api: &mut Y) -> Result<(ResourceAddress, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Bucket(self.receiver);
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
