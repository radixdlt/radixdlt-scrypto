use std::mem;
use native_sdk::resource::{ResourceManager, SysBucket};
use radix_engine_interface::api::api::{EngineApi, InvokableModel};
use crate::engine::{
    ApplicationError, CallFrameUpdate, ExecutableInvocation, Executor, LockFlags, RENode,
    ResolvedActor, ResolvedReceiver, ResolverApi, RuntimeError, SystemApi,
};
use crate::model::{BucketSubstate, Resource, ResourceOperationError};
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::types::{
    GlobalAddress, NativeFn, RENodeId, ResourceManagerOffset, SubstateOffset, WorktopFn,
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

impl<W: WasmEngine> ExecutableInvocation<W> for WorktopPutInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi<W>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Worktop;
        let mut call_frame_update = CallFrameUpdate::copy_ref(receiver);
        call_frame_update
            .nodes_to_move
            .push(RENodeId::Bucket(self.bucket.0));
        let actor = ResolvedActor::method(
            NativeFn::Worktop(WorktopFn::Put),
            ResolvedReceiver::new(receiver),
        );
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for WorktopPutInvocation {
    type Output = ();

    fn execute<Y>(self, api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + InvokableModel<RuntimeError>,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let resource_address = self.bucket.sys_resource_address(api)?;

        let mut substate_mut = api.get_ref_mut(worktop_handle)?;
        let worktop = substate_mut.worktop();


        if let Some(own) = worktop.resources.get(&resource_address).cloned() {
            let existing_bucket = Bucket(own.bucket_id());
            existing_bucket.sys_is_empty(api)?;
        } else {
            worktop.resources.insert(resource_address, Own::Bucket(self.bucket.0));
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl<W: WasmEngine> ExecutableInvocation<W> for WorktopTakeAmountInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi<W>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Worktop;
        let mut call_frame_update = CallFrameUpdate::copy_ref(receiver);
        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::Global(GlobalAddress::Resource(
                self.resource_address,
            )));
        let actor = ResolvedActor::method(
            NativeFn::Worktop(WorktopFn::TakeAmount),
            ResolvedReceiver::new(receiver),
        );
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for WorktopTakeAmountInvocation {
    type Output = Bucket;

    fn execute<Y>(self, api: &mut Y) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + InvokableModel<RuntimeError>,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = api.get_ref_mut(worktop_handle)?;
        let worktop = substate_mut.worktop();
        let bucket = if let Some(bucket) = worktop.resources.get(&self.resource_address).cloned() {
            bucket
        } else {
            let resman = ResourceManager(self.resource_address);
            let bucket = Own::Bucket(resman.new_empty_bucket(api)?.0);

            let mut substate_mut = api.get_ref_mut(worktop_handle)?;
            let worktop = substate_mut.worktop();
            worktop.resources.insert(self.resource_address, bucket);
            worktop.resources.get(&self.resource_address).cloned().unwrap()
        };

        let bucket = Bucket(bucket.bucket_id());
        let rtn_bucket = bucket.sys_take(self.amount, api)?;

        let update = CallFrameUpdate::move_node(RENodeId::Bucket(rtn_bucket.0));
        Ok((bucket, update))
    }
}

impl<W: WasmEngine> ExecutableInvocation<W> for WorktopTakeAllInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi<W>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Worktop;
        let mut call_frame_update = CallFrameUpdate::copy_ref(receiver);
        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::Global(GlobalAddress::Resource(
                self.resource_address,
            )));
        let actor = ResolvedActor::method(
            NativeFn::Worktop(WorktopFn::TakeAll),
            ResolvedReceiver::new(receiver),
        );
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for WorktopTakeAllInvocation {
    type Output = Bucket;

    fn execute<Y>(self, api: &mut Y) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + InvokableModel<RuntimeError>,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;
        let mut substate_mut = api.get_ref_mut(worktop_handle)?;
        let worktop = substate_mut.worktop();

        let rtn_bucket = if let Some(bucket) = worktop.resources.get(&self.resource_address) {
            let bucket = Bucket(bucket.bucket_id());
            let amount = bucket.sys_amount(api)?;
            let rtn_bucket = bucket.sys_take(amount, api)?;
            rtn_bucket
        } else {
            let resman = ResourceManager(self.resource_address);
            resman.new_empty_bucket(api)?
        };

        let update = CallFrameUpdate::move_node(RENodeId::Bucket(rtn_bucket.0));
        Ok((rtn_bucket, update))
    }
}

impl<W: WasmEngine> ExecutableInvocation<W> for WorktopTakeNonFungiblesInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi<W>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Worktop;
        let mut call_frame_update = CallFrameUpdate::copy_ref(receiver);
        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::Global(GlobalAddress::Resource(
                self.resource_address,
            )));
        let actor = ResolvedActor::method(
            NativeFn::Worktop(WorktopFn::TakeNonFungibles),
            ResolvedReceiver::new(receiver),
        );
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for WorktopTakeNonFungiblesInvocation {
    type Output = Bucket;

    fn execute<Y>(self, api: &mut Y) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + InvokableModel<RuntimeError>,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;
        let mut substate_mut = api.get_ref_mut(worktop_handle)?;
        let worktop = substate_mut.worktop();

        let bucket = if let Some(bucket) = worktop.resources.get(&self.resource_address).cloned() {
            bucket
        } else {
            let resman = ResourceManager(self.resource_address);
            let bucket = Own::Bucket(resman.new_empty_bucket(api)?.0);

            let mut substate_mut = api.get_ref_mut(worktop_handle)?;
            let worktop = substate_mut.worktop();
            worktop.resources.insert(self.resource_address, bucket);
            worktop.resources.get(&self.resource_address).cloned().unwrap()
        };

        let mut bucket = Bucket(bucket.bucket_id());
        let rtn_bucket = bucket.sys_take_non_fungibles(self.ids, api)?;
        let update = CallFrameUpdate::move_node(RENodeId::Bucket(rtn_bucket.0));
        Ok((bucket, update))
    }
}

impl<W: WasmEngine> ExecutableInvocation<W> for WorktopAssertContainsInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi<W>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Worktop;
        let mut call_frame_update = CallFrameUpdate::copy_ref(receiver);
        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::Global(GlobalAddress::Resource(
                self.resource_address,
            )));
        let actor = ResolvedActor::method(
            NativeFn::Worktop(WorktopFn::AssertContains),
            ResolvedReceiver::new(receiver),
        );
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for WorktopAssertContainsInvocation {
    type Output = ();

    fn execute<Y>(self, api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + InvokableModel<RuntimeError>,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle = api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate_ref = api.get_ref(worktop_handle)?;
        let worktop = substate_ref.worktop();

        let total_amount = if let Some(bucket) = worktop.resources.get(&self.resource_address).cloned() {
            Bucket(bucket.bucket_id()).sys_amount(api)?
        } else {
            Decimal::zero()
        };
        if total_amount.is_zero() {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::WorktopError(WorktopError::AssertionFailed),
            ));
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl<W: WasmEngine> ExecutableInvocation<W> for WorktopAssertContainsAmountInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi<W>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Worktop;
        let mut call_frame_update = CallFrameUpdate::copy_ref(receiver);
        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::Global(GlobalAddress::Resource(
                self.resource_address,
            )));
        let actor = ResolvedActor::method(
            NativeFn::Worktop(WorktopFn::AssertContainsAmount),
            ResolvedReceiver::new(receiver),
        );
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for WorktopAssertContainsAmountInvocation {
    type Output = ();

    fn execute<Y>(self, api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + InvokableModel<RuntimeError>,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle = api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate_ref = api.get_ref(worktop_handle)?;
        let worktop = substate_ref.worktop();
        let total_amount = if let Some(bucket) = worktop.resources.get(&self.resource_address).cloned() {
            Bucket(bucket.bucket_id()).sys_amount(api)?
        } else {
            Decimal::zero()
        };
        if total_amount < self.amount {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::WorktopError(WorktopError::AssertionFailed),
            ));
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl<W: WasmEngine> ExecutableInvocation<W> for WorktopAssertContainsNonFungiblesInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi<W>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Worktop;
        let mut call_frame_update = CallFrameUpdate::copy_ref(receiver);
        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::Global(GlobalAddress::Resource(
                self.resource_address,
            )));
        let actor = ResolvedActor::method(
            NativeFn::Worktop(WorktopFn::AssertContainsNonFungibles),
            ResolvedReceiver::new(receiver),
        );
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for WorktopAssertContainsNonFungiblesInvocation {
    type Output = ();

    fn execute<Y>(self, api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + InvokableModel<RuntimeError>,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle = api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate_ref = api.get_ref(worktop_handle)?;
        let worktop = substate_ref.worktop();

        let ids = if let Some(bucket) = worktop.resources.get(&self.resource_address) {
            let bucket = Bucket(bucket.bucket_id());
            bucket.sys_total_ids(api)?
        } else {
            BTreeSet::new()
        };
        if !ids
            .is_superset(&self.ids)
        {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::WorktopError(WorktopError::AssertionFailed),
            ));
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl<W: WasmEngine> ExecutableInvocation<W> for WorktopDrainInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi<W>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Worktop;
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = ResolvedActor::method(
            NativeFn::Worktop(WorktopFn::Drain),
            ResolvedReceiver::new(receiver),
        );
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for WorktopDrainInvocation {
    type Output = Vec<Bucket>;

    fn execute<Y>(self, api: &mut Y) -> Result<(Vec<Bucket>, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi+ InvokableModel<RuntimeError>,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;
        let mut buckets = Vec::new();
        let mut nodes_to_move = Vec::new();
        let mut substate_mut = api.get_ref_mut(worktop_handle)?;
        let worktop = substate_mut.worktop();
        let resources = mem::replace(&mut worktop.resources, BTreeMap::new());
        for (_, resource) in resources {
            let bucket = Bucket(resource.bucket_id());
            nodes_to_move.push(RENodeId::Bucket(bucket.0));
            buckets.push(bucket);
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
