use crate::engine::{
    ApplicationError, CallFrameUpdate, LockFlags, NativeExecutable, NativeInvocation,
    NativeInvocationInfo, RENode, RuntimeError, SystemApi,
};
use crate::model::{BucketSubstate, Resource, ResourceOperationError};
use crate::types::*;
use radix_engine_lib::engine::types::{
    GlobalAddress, NativeMethod, RENodeId, ResourceManagerOffset, SubstateOffset, WorktopMethod,
    WorktopOffset,
};
use radix_engine_lib::resource::{
    WorktopAssertContainsAmountInvocation, WorktopAssertContainsInvocation,
    WorktopAssertContainsNonFungiblesInvocation, WorktopDrainInvocation, WorktopPutInvocation,
    WorktopTakeAllInvocation, WorktopTakeAmountInvocation, WorktopTakeNonFungiblesInvocation,
};

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

impl NativeExecutable for WorktopPutInvocation {
    type NativeOutput = ();

    fn execute<Y>(input: Self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let bucket = system_api
            .drop_node(RENodeId::Bucket(input.bucket.0))?
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

impl NativeInvocation for WorktopPutInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Worktop(WorktopMethod::Put),
            RENodeId::Worktop,
            CallFrameUpdate::move_node(RENodeId::Bucket(self.bucket.0)),
        )
    }
}

impl NativeExecutable for WorktopTakeAmountInvocation {
    type NativeOutput = radix_engine_lib::resource::Bucket;

    fn execute<Y>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(radix_engine_lib::resource::Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let maybe_resource = {
            let mut substate_mut = system_api.get_ref_mut(worktop_handle)?;
            let worktop = substate_mut.worktop();
            let maybe_resource =
                worktop
                    .take(input.amount, input.resource_address)
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
                let resource_id = RENodeId::Global(GlobalAddress::Resource(input.resource_address));
                let offset =
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                let resource_handle =
                    system_api.lock_substate(resource_id, offset, LockFlags::read_only())?;
                let substate_ref = system_api.get_ref(resource_handle)?;
                substate_ref.resource_manager().resource_type
            };

            Resource::new_empty(input.resource_address, resource_type)
        };
        let bucket_id = system_api
            .create_node(RENode::Bucket(BucketSubstate::new(resource_resource)))?
            .into();
        Ok((
            radix_engine_lib::resource::Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

impl NativeInvocation for WorktopTakeAmountInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Worktop(WorktopMethod::TakeAmount),
            RENodeId::Worktop,
            CallFrameUpdate::copy_ref(RENodeId::Global(GlobalAddress::Resource(
                self.resource_address,
            ))),
        )
    }
}

impl NativeExecutable for WorktopTakeAllInvocation {
    type NativeOutput = radix_engine_lib::resource::Bucket;

    fn execute<Y>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(radix_engine_lib::resource::Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let maybe_resource = {
            let mut substate_mut = system_api.get_ref_mut(worktop_handle)?;
            let worktop = substate_mut.worktop();
            let maybe_resource = worktop.take_all(input.resource_address).map_err(|e| {
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
                let resource_id = RENodeId::Global(GlobalAddress::Resource(input.resource_address));
                let offset =
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                let resource_handle =
                    system_api.lock_substate(resource_id, offset, LockFlags::read_only())?;
                let substate_ref = system_api.get_ref(resource_handle)?;
                substate_ref.resource_manager().resource_type
            };

            Resource::new_empty(input.resource_address, resource_type)
        };

        let bucket_id = system_api
            .create_node(RENode::Bucket(BucketSubstate::new(resource_resource)))?
            .into();

        Ok((
            radix_engine_lib::resource::Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

impl NativeInvocation for WorktopTakeAllInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Worktop(WorktopMethod::TakeAll),
            RENodeId::Worktop,
            CallFrameUpdate::copy_ref(RENodeId::Global(GlobalAddress::Resource(
                self.resource_address,
            ))),
        )
    }
}

impl NativeExecutable for WorktopTakeNonFungiblesInvocation {
    type NativeOutput = radix_engine_lib::resource::Bucket;

    fn execute<Y>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(radix_engine_lib::resource::Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let maybe_resource = {
            let mut substate_mut = system_api.get_ref_mut(worktop_handle)?;
            let worktop = substate_mut.worktop();
            let maybe_resource = worktop
                .take_non_fungibles(&input.ids, input.resource_address)
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
                let resource_id = RENodeId::Global(GlobalAddress::Resource(input.resource_address));
                let offset =
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                let resource_handle =
                    system_api.lock_substate(resource_id, offset, LockFlags::read_only())?;
                let substate_ref = system_api.get_ref(resource_handle)?;
                substate_ref.resource_manager().resource_type
            };

            Resource::new_empty(input.resource_address, resource_type)
        };

        let bucket_id = system_api
            .create_node(RENode::Bucket(BucketSubstate::new(resource_resource)))?
            .into();

        Ok((
            radix_engine_lib::resource::Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

impl NativeInvocation for WorktopTakeNonFungiblesInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Worktop(WorktopMethod::TakeNonFungibles),
            RENodeId::Worktop,
            CallFrameUpdate::copy_ref(RENodeId::Global(GlobalAddress::Resource(
                self.resource_address,
            ))),
        )
    }
}

impl NativeExecutable for WorktopAssertContainsInvocation {
    type NativeOutput = ();

    fn execute<Y>(input: Self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate_ref = system_api.get_ref(worktop_handle)?;
        let worktop = substate_ref.worktop();
        if worktop.total_amount(input.resource_address).is_zero() {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::WorktopError(WorktopError::AssertionFailed),
            ));
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for WorktopAssertContainsInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Worktop(WorktopMethod::AssertContains),
            RENodeId::Worktop,
            CallFrameUpdate::copy_ref(RENodeId::Global(GlobalAddress::Resource(
                self.resource_address,
            ))),
        )
    }
}

impl NativeExecutable for WorktopAssertContainsAmountInvocation {
    type NativeOutput = ();

    fn execute<Y>(input: Self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate_ref = system_api.get_ref(worktop_handle)?;
        let worktop = substate_ref.worktop();
        if worktop.total_amount(input.resource_address) < input.amount {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::WorktopError(WorktopError::AssertionFailed),
            ));
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for WorktopAssertContainsAmountInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Worktop(WorktopMethod::AssertContainsAmount),
            RENodeId::Worktop,
            CallFrameUpdate::copy_ref(RENodeId::Global(GlobalAddress::Resource(
                self.resource_address,
            ))),
        )
    }
}

impl NativeExecutable for WorktopAssertContainsNonFungiblesInvocation {
    type NativeOutput = ();

    fn execute<Y>(input: Self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate_ref = system_api.get_ref(worktop_handle)?;
        let worktop = substate_ref.worktop();
        if !worktop
            .total_ids(input.resource_address)
            .map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::WorktopError(
                    WorktopError::ResourceOperationError(e),
                ))
            })?
            .is_superset(&input.ids)
        {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::WorktopError(WorktopError::AssertionFailed),
            ));
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for WorktopAssertContainsNonFungiblesInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Worktop(WorktopMethod::AssertContainsNonFungibles),
            RENodeId::Worktop,
            CallFrameUpdate::copy_ref(RENodeId::Global(GlobalAddress::Resource(
                self.resource_address,
            ))),
        )
    }
}

impl NativeExecutable for WorktopDrainInvocation {
    type NativeOutput = Vec<radix_engine_lib::resource::Bucket>;

    fn execute<Y>(
        _input: Self,
        system_api: &mut Y,
    ) -> Result<(Vec<radix_engine_lib::resource::Bucket>, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut resources = Vec::new();
        {
            let mut substate_mut = system_api.get_ref_mut(worktop_handle)?;
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
            let bucket_id = system_api
                .create_node(RENode::Bucket(BucketSubstate::new(resource)))?
                .into();
            buckets.push(radix_engine_lib::resource::Bucket(bucket_id));
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

impl NativeInvocation for WorktopDrainInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Worktop(WorktopMethod::Drain),
            RENodeId::Worktop,
            CallFrameUpdate::empty(),
        )
    }
}
