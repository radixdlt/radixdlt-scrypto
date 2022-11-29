use crate::engine::{
    ApplicationError, CallFrameUpdate, ExecutableInvocation, LockFlags, MethodDeref,
    NativeExecutor, NativeProcedure, REActor, RENode, ResolvedMethod, ResolvedReceiver,
    RuntimeError, SystemApi,
};
use crate::model::{BucketSubstate, Resource, ResourceOperationError};
use crate::types::*;
use radix_engine_interface::api::types::{
    GlobalAddress, NativeMethod, RENodeId, ResourceManagerOffset, SubstateOffset, WorktopMethod,
    WorktopOffset,
};
use radix_engine_interface::model::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub enum WorktopError {
    InvalidRequestData(DecodeError),
    MethodNotFound(String),
    ResourceOperationError(ResourceOperationError),
    ResourceNotFound(ResourceAddress),
    CouldNotCreateBucket,
    CouldNotTakeBucket,
    AssertionFailed,
    CouldNotDrop,
}

impl ExecutableInvocation for WorktopPutInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Worktop;
        let mut call_frame_update = CallFrameUpdate::copy_ref(receiver);
        call_frame_update
            .nodes_to_move
            .push(RENodeId::Bucket(self.bucket.0));
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Worktop(WorktopMethod::Put)),
            ResolvedReceiver::new(receiver),
        );
        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for WorktopPutInvocation {
    type Output = ();

    fn main<Y>(self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let bucket = system_api
            .drop_node(RENodeId::Bucket(self.bucket.0))?
            .into();
        let mut substate_mut = system_api.get_ref_mut(worktop_handle)?;
        let worktop = substate_mut.worktop();
        worktop.put(bucket).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::WorktopError(
                WorktopError::ResourceOperationError(e),
            ))
        })?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for WorktopTakeAmountInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Worktop;
        let mut call_frame_update = CallFrameUpdate::copy_ref(receiver);
        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::Global(GlobalAddress::Resource(
                self.resource_address,
            )));
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Worktop(WorktopMethod::TakeAmount)),
            ResolvedReceiver::new(receiver),
        );
        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for WorktopTakeAmountInvocation {
    type Output = Bucket;

    fn main<Y>(self, api: &mut Y) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let maybe_resource = {
            let mut substate_mut = api.get_ref_mut(worktop_handle)?;
            let worktop = substate_mut.worktop();
            let maybe_resource = worktop
                .take(self.amount, self.resource_address)
                .map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::WorktopError(
                        WorktopError::ResourceOperationError(e),
                    ))
                })?;
            maybe_resource
        };

        let resource_resource = if let Some(resource) = maybe_resource {
            resource
        } else {
            let resource_type = {
                let resource_id = RENodeId::Global(GlobalAddress::Resource(self.resource_address));
                let offset =
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                let resource_handle =
                    api.lock_substate(resource_id, offset, LockFlags::read_only())?;
                let substate_ref = api.get_ref(resource_handle)?;
                substate_ref.resource_manager().resource_type
            };

            Resource::new_empty(self.resource_address, resource_type)
        };
        let node_id = api.allocate_node_id(RENodeType::Bucket)?;
        api.create_node(
            node_id,
            RENode::Bucket(BucketSubstate::new(resource_resource)),
        )?;
        let bucket_id = node_id.into();

        Ok((
            Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

impl ExecutableInvocation for WorktopTakeAllInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Worktop;
        let mut call_frame_update = CallFrameUpdate::copy_ref(receiver);
        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::Global(GlobalAddress::Resource(
                self.resource_address,
            )));
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Worktop(WorktopMethod::TakeAll)),
            ResolvedReceiver::new(receiver),
        );
        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for WorktopTakeAllInvocation {
    type Output = Bucket;

    fn main<Y>(self, api: &mut Y) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let maybe_resource = {
            let mut substate_mut = api.get_ref_mut(worktop_handle)?;
            let worktop = substate_mut.worktop();
            let maybe_resource = worktop.take_all(self.resource_address).map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::WorktopError(
                    WorktopError::ResourceOperationError(e),
                ))
            })?;
            maybe_resource
        };

        let resource_resource = if let Some(resource) = maybe_resource {
            resource
        } else {
            let resource_type = {
                let resource_id = RENodeId::Global(GlobalAddress::Resource(self.resource_address));
                let offset =
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                let resource_handle =
                    api.lock_substate(resource_id, offset, LockFlags::read_only())?;
                let substate_ref = api.get_ref(resource_handle)?;
                substate_ref.resource_manager().resource_type
            };

            Resource::new_empty(self.resource_address, resource_type)
        };

        let node_id = api.allocate_node_id(RENodeType::Bucket)?;
        api.create_node(
            node_id,
            RENode::Bucket(BucketSubstate::new(resource_resource)),
        )?;
        let bucket_id = node_id.into();

        Ok((
            Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

impl ExecutableInvocation for WorktopTakeNonFungiblesInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Worktop;
        let mut call_frame_update = CallFrameUpdate::copy_ref(receiver);
        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::Global(GlobalAddress::Resource(
                self.resource_address,
            )));
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Worktop(WorktopMethod::TakeNonFungibles)),
            ResolvedReceiver::new(receiver),
        );
        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for WorktopTakeNonFungiblesInvocation {
    type Output = Bucket;

    fn main<Y>(self, api: &mut Y) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let maybe_resource = {
            let mut substate_mut = api.get_ref_mut(worktop_handle)?;
            let worktop = substate_mut.worktop();
            let maybe_resource = worktop
                .take_non_fungibles(&self.ids, self.resource_address)
                .map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::WorktopError(
                        WorktopError::ResourceOperationError(e),
                    ))
                })?;
            maybe_resource
        };

        let resource_resource = if let Some(resource) = maybe_resource {
            resource
        } else {
            let resource_type = {
                let resource_id = RENodeId::Global(GlobalAddress::Resource(self.resource_address));
                let offset =
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                let resource_handle =
                    api.lock_substate(resource_id, offset, LockFlags::read_only())?;
                let substate_ref = api.get_ref(resource_handle)?;
                substate_ref.resource_manager().resource_type
            };

            Resource::new_empty(self.resource_address, resource_type)
        };

        let node_id = api.allocate_node_id(RENodeType::Bucket)?;
        api.create_node(
            node_id,
            RENode::Bucket(BucketSubstate::new(resource_resource)),
        )?;
        let bucket_id = node_id.into();

        Ok((
            Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

impl ExecutableInvocation for WorktopAssertContainsInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Worktop;
        let mut call_frame_update = CallFrameUpdate::copy_ref(receiver);
        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::Global(GlobalAddress::Resource(
                self.resource_address,
            )));
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Worktop(WorktopMethod::AssertContains)),
            ResolvedReceiver::new(receiver),
        );
        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for WorktopAssertContainsInvocation {
    type Output = ();

    fn main<Y>(self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate_ref = system_api.get_ref(worktop_handle)?;
        let worktop = substate_ref.worktop();
        if worktop.total_amount(self.resource_address).is_zero() {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::WorktopError(WorktopError::AssertionFailed),
            ));
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for WorktopAssertContainsAmountInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Worktop;
        let mut call_frame_update = CallFrameUpdate::copy_ref(receiver);
        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::Global(GlobalAddress::Resource(
                self.resource_address,
            )));
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Worktop(WorktopMethod::AssertContainsAmount)),
            ResolvedReceiver::new(receiver),
        );
        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for WorktopAssertContainsAmountInvocation {
    type Output = ();

    fn main<Y>(self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate_ref = system_api.get_ref(worktop_handle)?;
        let worktop = substate_ref.worktop();
        if worktop.total_amount(self.resource_address) < self.amount {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::WorktopError(WorktopError::AssertionFailed),
            ));
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for WorktopAssertContainsNonFungiblesInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Worktop;
        let mut call_frame_update = CallFrameUpdate::copy_ref(receiver);
        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::Global(GlobalAddress::Resource(
                self.resource_address,
            )));
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Worktop(
                WorktopMethod::AssertContainsNonFungibles,
            )),
            ResolvedReceiver::new(receiver),
        );
        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for WorktopAssertContainsNonFungiblesInvocation {
    type Output = ();

    fn main<Y>(self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate_ref = system_api.get_ref(worktop_handle)?;
        let worktop = substate_ref.worktop();
        if !worktop
            .total_ids(self.resource_address)
            .map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::WorktopError(
                    WorktopError::ResourceOperationError(e),
                ))
            })?
            .is_superset(&self.ids)
        {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::WorktopError(WorktopError::AssertionFailed),
            ));
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for WorktopDrainInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Worktop;
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Worktop(WorktopMethod::Drain)),
            ResolvedReceiver::new(receiver),
        );
        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for WorktopDrainInvocation {
    type Output = Vec<Bucket>;

    fn main<Y>(self, api: &mut Y) -> Result<(Vec<Bucket>, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut resources = Vec::new();
        {
            let mut substate_mut = api.get_ref_mut(worktop_handle)?;
            let worktop = substate_mut.worktop();
            for (_, resource) in worktop.resources.drain() {
                let taken = resource.borrow_mut().take_all_liquid().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::WorktopError(
                        WorktopError::ResourceOperationError(e),
                    ))
                })?;
                if !taken.is_empty() {
                    resources.push(taken);
                }
            }
        }

        let mut buckets = Vec::new();
        let mut nodes_to_move = Vec::new();
        for resource in resources {
            let node_id = api.allocate_node_id(RENodeType::Bucket)?;
            api.create_node(node_id, RENode::Bucket(BucketSubstate::new(resource)))?;
            let bucket_id = node_id.into();
            buckets.push(Bucket(bucket_id));
            nodes_to_move.push(RENodeId::Bucket(bucket_id));
        }

        Ok((
            buckets,
            CallFrameUpdate {
                nodes_to_move,
                node_refs_to_copy: HashSet::new(),
            },
        ))
    }
}
