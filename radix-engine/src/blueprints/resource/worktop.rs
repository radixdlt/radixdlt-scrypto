use crate::errors::{ApplicationError, InterpreterError};
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::kernel::kernel_api::LockFlags;
use crate::kernel::KernelNodeApi;
use crate::kernel::{
    CallFrameUpdate, ExecutableInvocation, Executor, ResolvedActor, ResolvedReceiver,
};
use crate::types::*;
use crate::wasm::WasmEngine;
use native_sdk::resource::{ResourceManager, SysBucket};
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::types::{
    GlobalAddress, NativeFn, RENodeId, SubstateOffset, WorktopFn, WorktopOffset,
};
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::api::ClientDerefApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::ScryptoValue;

#[derive(Debug)]
pub struct WorktopSubstate {
    pub resources: BTreeMap<ResourceAddress, Own>,
}

impl WorktopSubstate {
    pub fn new() -> Self {
        Self {
            resources: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum WorktopError {
    AssertionFailed,
}

pub struct WorktopBlueprint;

impl WorktopBlueprint {
    pub(crate) fn put<Y>(
        _ignored: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: KernelNodeApi
            + KernelSubstateApi
            + ClientApi<RuntimeError>,
    {
        let input: WorktopPutInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let worktop_handle =
            api.lock_substate(
                RENodeId::Worktop,
                NodeModuleId::SELF,
                SubstateOffset::Worktop(WorktopOffset::Worktop),
                LockFlags::MUTABLE,
            )?;

        let resource_address = input.bucket.sys_resource_address(api)?;

        let mut substate_mut = api.get_ref_mut(worktop_handle)?;
        let worktop = substate_mut.worktop();

        if let Some(own) = worktop.resources.get(&resource_address).cloned() {
            let existing_bucket = Bucket(own.bucket_id());
            existing_bucket.sys_put(input.bucket, api)?;
        } else {
            worktop
                .resources
                .insert(resource_address, Own::Bucket(input.bucket.0));
        }

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn take<Y>(
        _ignored: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: KernelNodeApi
            + KernelSubstateApi
            + ClientApi<RuntimeError>,
    {
        let input: WorktopTakeInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let worktop_handle =
            api.lock_substate(
                RENodeId::Worktop,
                NodeModuleId::SELF,
                SubstateOffset::Worktop(WorktopOffset::Worktop),
                LockFlags::MUTABLE,
            )?;

        let mut substate_mut = api.get_ref_mut(worktop_handle)?;
        let worktop = substate_mut.worktop();
        let bucket = if let Some(bucket) = worktop.resources.get(&input.resource_address).cloned() {
            bucket
        } else {
            let resman = ResourceManager(input.resource_address);
            let bucket = Own::Bucket(resman.new_empty_bucket(api)?.0);

            let mut substate_mut = api.get_ref_mut(worktop_handle)?;
            let worktop = substate_mut.worktop();
            worktop.resources.insert(input.resource_address, bucket);
            worktop
                .resources
                .get(&input.resource_address)
                .cloned()
                .unwrap()
        };

        let rtn_bucket = Bucket(bucket.bucket_id()).sys_take(input.amount, api)?;

        Ok(IndexedScryptoValue::from_typed(&rtn_bucket))
    }

    pub(crate) fn take_all<Y>(
        _ignored: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: KernelNodeApi
            + KernelSubstateApi
            + ClientApi<RuntimeError>,
    {
        let input: WorktopTakeAllInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let worktop_handle =
            api.lock_substate(
                RENodeId::Worktop,
                NodeModuleId::SELF,
                SubstateOffset::Worktop(WorktopOffset::Worktop),
                LockFlags::MUTABLE,
            )?;
        let mut substate_mut = api.get_ref_mut(worktop_handle)?;
        let worktop = substate_mut.worktop();

        let rtn_bucket = if let Some(bucket) = worktop.resources.get(&input.resource_address) {
            let bucket = Bucket(bucket.bucket_id());
            let amount = bucket.sys_amount(api)?;
            let rtn_bucket = bucket.sys_take(amount, api)?;
            rtn_bucket
        } else {
            let resman = ResourceManager(input.resource_address);
            resman.new_empty_bucket(api)?
        };

        Ok(IndexedScryptoValue::from_typed(&rtn_bucket))
    }

    pub(crate) fn take_non_fungibles<Y>(
        _ignored: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: KernelNodeApi
            + KernelSubstateApi
            + ClientApi<RuntimeError>,
    {
        let input: WorktopTakeNonFungiblesInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let worktop_handle =
            api.lock_substate(
                RENodeId::Worktop,
                NodeModuleId::SELF,
                SubstateOffset::Worktop(WorktopOffset::Worktop),
                LockFlags::MUTABLE,
            )?;
        let mut substate_mut = api.get_ref_mut(worktop_handle)?;
        let worktop = substate_mut.worktop();

        let bucket = if let Some(bucket) = worktop.resources.get(&input.resource_address).cloned() {
            bucket
        } else {
            let resman = ResourceManager(input.resource_address);
            let bucket = Own::Bucket(resman.new_empty_bucket(api)?.0);

            let mut substate_mut = api.get_ref_mut(worktop_handle)?;
            let worktop = substate_mut.worktop();
            worktop.resources.insert(input.resource_address, bucket);
            worktop
                .resources
                .get(&input.resource_address)
                .cloned()
                .unwrap()
        };

        let mut bucket = Bucket(bucket.bucket_id());
        let rtn_bucket = bucket.sys_take_non_fungibles(input.ids, api)?;

        Ok(IndexedScryptoValue::from_typed(&rtn_bucket))
    }
}

impl ExecutableInvocation for WorktopAssertContainsInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
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

    fn execute<Y, W: WasmEngine>(self, api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle =
            api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?;

        let substate_ref = api.get_ref(worktop_handle)?;
        let worktop = substate_ref.worktop();

        let total_amount =
            if let Some(bucket) = worktop.resources.get(&self.resource_address).cloned() {
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

impl ExecutableInvocation for WorktopAssertContainsAmountInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
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

    fn execute<Y, W: WasmEngine>(self, api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle =
            api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?;

        let substate_ref = api.get_ref(worktop_handle)?;
        let worktop = substate_ref.worktop();
        let total_amount =
            if let Some(bucket) = worktop.resources.get(&self.resource_address).cloned() {
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

impl ExecutableInvocation for WorktopAssertContainsNonFungiblesInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
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

    fn execute<Y, W: WasmEngine>(self, api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle =
            api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?;

        let substate_ref = api.get_ref(worktop_handle)?;
        let worktop = substate_ref.worktop();

        let ids = if let Some(bucket) = worktop.resources.get(&self.resource_address) {
            let bucket = Bucket(bucket.bucket_id());
            bucket.sys_total_ids(api)?
        } else {
            BTreeSet::new()
        };
        if !ids.is_superset(&self.ids) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::WorktopError(WorktopError::AssertionFailed),
            ));
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for WorktopDrainInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
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

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Vec<Bucket>, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let node_id = RENodeId::Worktop;
        let offset = SubstateOffset::Worktop(WorktopOffset::Worktop);
        let worktop_handle =
            api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::MUTABLE)?;
        let mut buckets = Vec::new();
        let mut nodes_to_move = Vec::new();
        let mut substate_mut = api.get_ref_mut(worktop_handle)?;
        let worktop = substate_mut.worktop();
        let bucket_ids: Vec<BucketId> = worktop
            .resources
            .iter()
            .map(|(_, own)| own.bucket_id())
            .collect();
        for bucket_id in bucket_ids {
            let bucket = Bucket(bucket_id);
            let amount = bucket.sys_amount(api)?;
            let bucket = bucket.sys_take(amount, api)?;
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
