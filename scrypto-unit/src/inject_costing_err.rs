use radix_engine::errors::{RuntimeError, SystemModuleError};
use radix_engine::kernel::call_frame::{CallFrameMessage, NodeVisibility};
use radix_engine::kernel::kernel_api::{
    DroppedNode, KernelApi, KernelInternalApi, KernelInvocation, KernelInvokeApi, KernelNodeApi,
    KernelSubstateApi, SystemState,
};
use radix_engine::kernel::kernel_callback_api::{
    CloseSubstateEvent, CreateNodeEvent, DrainSubstatesEvent, DropNodeEvent, KernelCallbackObject,
    MoveModuleEvent, OpenSubstateEvent, ReadSubstateEvent, RemoveSubstateEvent, ScanKeysEvent,
    ScanSortedSubstatesEvent, SetSubstateEvent, WriteSubstateEvent,
};
use radix_engine::system::system_callback::SystemConfig;
use radix_engine::system::system_callback_api::SystemCallbackObject;
use radix_engine::system::system_modules::costing::{CostingError, FeeReserveError, OnApplyCost};
use radix_engine::system::system_modules::execution_trace::{BucketSnapshot, ProofSnapshot};
use radix_engine::track::NodeSubstates;
use radix_engine::transaction::WrappedSystem;
use radix_engine::types::*;
use radix_engine::vm::wasm::DefaultWasmEngine;
use radix_engine::vm::Vm;
use radix_engine_store_interface::db_key_mapper::SubstateKeyContent;
use transaction::prelude::PreAllocatedAddress;

pub type InjectSystemCostingError<'a, E> =
    InjectCostingError<SystemConfig<Vm<'a, DefaultWasmEngine, E>>>;

pub struct InjectCostingError<K: KernelCallbackObject> {
    fail_after: Rc<RefCell<u64>>,
    callback_object: K,
}

impl<C: SystemCallbackObject> WrappedSystem<C> for InjectCostingError<SystemConfig<C>> {
    type Init = u64;

    fn create(mut config: SystemConfig<C>, error_after_count: u64) -> Self {
        let fail_after = Rc::new(RefCell::new(error_after_count));
        config.modules.costing_mut().unwrap().on_apply_cost = OnApplyCost::ForceFailOnCount {
            fail_after: fail_after.clone(),
        };

        Self {
            fail_after,
            callback_object: config,
        }
    }

    fn system_mut(&mut self) -> &mut SystemConfig<C> {
        &mut self.callback_object
    }

    fn to_system(self) -> SystemConfig<C> {
        self.callback_object
    }
}

impl<K: KernelCallbackObject> InjectCostingError<K> {
    fn maybe_err(&mut self) -> Result<(), RuntimeError> {
        if *self.fail_after.borrow() == 0 {
            return Ok(());
        }

        *self.fail_after.borrow_mut() -= 1;

        if *self.fail_after.borrow() == 0 {
            return Err(RuntimeError::SystemModuleError(
                SystemModuleError::CostingError(CostingError::FeeReserveError(
                    FeeReserveError::InsufficientBalance {
                        required: Decimal::MAX,
                        remaining: Decimal::ONE,
                    },
                )),
            ));
        }

        Ok(())
    }
}

macro_rules! wrapped_api {
    ($api:ident) => {
        WrappedKernelApi {
            api: $api,
            phantom: PhantomData::default(),
        }
    };
}

macro_rules! wrapped_internal_api {
    ($api:ident) => {
        WrappedKernelInternalApi {
            api: $api,
            phantom: PhantomData::default(),
        }
    };
}

impl<'a, K: KernelCallbackObject + 'a> KernelCallbackObject for InjectCostingError<K> {
    type LockData = K::LockData;
    type CallFrameData = K::CallFrameData;

    fn on_init<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        let mut api = wrapped_api!(api);
        K::on_init(&mut api)
    }

    fn start<Y>(
        api: &mut Y,
        manifest_encoded_instructions: &[u8],
        pre_allocated_addresses: &Vec<PreAllocatedAddress>,
        references: &IndexSet<Reference>,
        blobs: &IndexMap<Hash, Vec<u8>>,
    ) -> Result<Vec<u8>, RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        let mut api = wrapped_api!(api);
        K::start(
            &mut api,
            manifest_encoded_instructions,
            pre_allocated_addresses,
            references,
            blobs,
        )
    }

    fn on_teardown<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        K::on_teardown(&mut api)
    }

    fn on_pin_node(&mut self, node_id: &NodeId) -> Result<(), RuntimeError> {
        self.maybe_err()?;
        self.callback_object.on_pin_node(node_id)
    }

    fn on_create_node<Y>(api: &mut Y, event: CreateNodeEvent) -> Result<(), RuntimeError>
    where
        Y: KernelInternalApi<Self>,
    {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        K::on_create_node(&mut api, event)
    }

    fn on_drop_node<Y>(api: &mut Y, event: DropNodeEvent) -> Result<(), RuntimeError>
    where
        Y: KernelInternalApi<Self>,
    {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        K::on_drop_node(&mut api, event)
    }

    fn on_move_module<Y>(api: &mut Y, event: MoveModuleEvent) -> Result<(), RuntimeError>
    where
        Y: KernelInternalApi<Self>,
    {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        K::on_move_module(&mut api, event)
    }

    fn on_open_substate<Y>(api: &mut Y, event: OpenSubstateEvent) -> Result<(), RuntimeError>
    where
        Y: KernelInternalApi<Self>,
    {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        K::on_open_substate(&mut api, event)
    }

    fn on_close_substate<Y>(api: &mut Y, event: CloseSubstateEvent) -> Result<(), RuntimeError>
    where
        Y: KernelInternalApi<Self>,
    {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        K::on_close_substate(&mut api, event)
    }

    fn on_read_substate<Y>(api: &mut Y, event: ReadSubstateEvent) -> Result<(), RuntimeError>
    where
        Y: KernelInternalApi<Self>,
    {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        K::on_read_substate(&mut api, event)
    }

    fn on_write_substate<Y>(api: &mut Y, event: WriteSubstateEvent) -> Result<(), RuntimeError>
    where
        Y: KernelInternalApi<Self>,
    {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        K::on_write_substate(&mut api, event)
    }

    fn on_set_substate(&mut self, event: SetSubstateEvent) -> Result<(), RuntimeError> {
        self.maybe_err()?;
        self.callback_object.on_set_substate(event)
    }

    fn on_remove_substate(&mut self, event: RemoveSubstateEvent) -> Result<(), RuntimeError> {
        self.maybe_err()?;
        self.callback_object.on_remove_substate(event)
    }

    fn on_scan_keys(&mut self, event: ScanKeysEvent) -> Result<(), RuntimeError> {
        self.maybe_err()?;
        self.callback_object.on_scan_keys(event)
    }

    fn on_drain_substates(&mut self, event: DrainSubstatesEvent) -> Result<(), RuntimeError> {
        self.maybe_err()?;
        self.callback_object.on_drain_substates(event)
    }

    fn on_scan_sorted_substates(
        &mut self,
        event: ScanSortedSubstatesEvent,
    ) -> Result<(), RuntimeError> {
        self.maybe_err()?;
        self.callback_object.on_scan_sorted_substates(event)
    }

    fn before_invoke<Y>(
        invocation: &KernelInvocation<Self::CallFrameData>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        K::before_invoke(invocation, &mut api)
    }

    fn after_invoke<Y>(output: &IndexedScryptoValue, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        K::after_invoke(output, &mut api)
    }

    fn on_execution_start<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        K::on_execution_start(&mut api)
    }

    fn on_execution_finish<Y>(message: &CallFrameMessage, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        K::on_execution_finish(message, &mut api)
    }

    fn on_allocate_node_id<Y>(entity_type: EntityType, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        K::on_allocate_node_id(entity_type, &mut api)
    }

    fn invoke_upstream<Y>(
        args: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        K::invoke_upstream(args, &mut api)
    }

    fn auto_drop<Y>(nodes: Vec<NodeId>, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        K::auto_drop(nodes, &mut api)
    }

    fn on_mark_substate_as_transient(
        &mut self,
        node_id: &NodeId,
        partition_number: &PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Result<(), RuntimeError> {
        self.maybe_err()?;
        self.callback_object
            .on_mark_substate_as_transient(node_id, partition_number, substate_key)
    }

    fn on_substate_lock_fault<Y>(
        node_id: NodeId,
        partition_num: PartitionNumber,
        offset: &SubstateKey,
        api: &mut Y,
    ) -> Result<bool, RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        K::on_substate_lock_fault(node_id, partition_num, offset, &mut api)
    }

    fn on_drop_node_mut<Y>(node_id: &NodeId, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        K::on_drop_node_mut(node_id, &mut api)
    }

    fn on_move_node<Y>(
        node_id: &NodeId,
        is_moving_down: bool,
        is_to_barrier: bool,
        destination_blueprint_id: Option<BlueprintId>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        K::on_move_node(
            node_id,
            is_moving_down,
            is_to_barrier,
            destination_blueprint_id,
            &mut api,
        )
    }
}

pub struct WrappedKernelApi<'a, M: KernelCallbackObject + 'a, K: KernelApi<InjectCostingError<M>>> {
    api: &'a mut K,
    phantom: PhantomData<M>,
}

impl<'a, M: KernelCallbackObject, K: KernelApi<InjectCostingError<M>>> KernelNodeApi
    for WrappedKernelApi<'a, M, K>
{
    fn kernel_pin_node(&mut self, node_id: NodeId) -> Result<(), RuntimeError> {
        self.api.kernel_pin_node(node_id)
    }

    fn kernel_allocate_node_id(&mut self, entity_type: EntityType) -> Result<NodeId, RuntimeError> {
        self.api.kernel_allocate_node_id(entity_type)
    }

    fn kernel_create_node(
        &mut self,
        node_id: NodeId,
        node_substates: NodeSubstates,
    ) -> Result<(), RuntimeError> {
        self.api.kernel_create_node(node_id, node_substates)
    }

    fn kernel_create_node_from(
        &mut self,
        node_id: NodeId,
        partitions: BTreeMap<PartitionNumber, (NodeId, PartitionNumber)>,
    ) -> Result<(), RuntimeError> {
        self.api.kernel_create_node_from(node_id, partitions)
    }

    fn kernel_drop_node(&mut self, node_id: &NodeId) -> Result<DroppedNode, RuntimeError> {
        self.api.kernel_drop_node(node_id)
    }
}

impl<'a, M: KernelCallbackObject, Y: KernelApi<InjectCostingError<M>>>
    KernelSubstateApi<M::LockData> for WrappedKernelApi<'a, M, Y>
{
    fn kernel_mark_substate_as_transient(
        &mut self,
        node_id: NodeId,
        partition_num: PartitionNumber,
        key: SubstateKey,
    ) -> Result<(), RuntimeError> {
        self.api
            .kernel_mark_substate_as_transient(node_id, partition_num, key)
    }

    fn kernel_open_substate_with_default<F: FnOnce() -> IndexedScryptoValue>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        default: Option<F>,
        lock_data: M::LockData,
    ) -> Result<SubstateHandle, RuntimeError> {
        self.api.kernel_open_substate_with_default(
            node_id,
            partition_num,
            substate_key,
            flags,
            default,
            lock_data,
        )
    }

    fn kernel_get_lock_data(
        &mut self,
        lock_handle: SubstateHandle,
    ) -> Result<M::LockData, RuntimeError> {
        self.api.kernel_get_lock_data(lock_handle)
    }

    fn kernel_close_substate(&mut self, lock_handle: SubstateHandle) -> Result<(), RuntimeError> {
        self.api.kernel_close_substate(lock_handle)
    }

    fn kernel_read_substate(
        &mut self,
        lock_handle: SubstateHandle,
    ) -> Result<&IndexedScryptoValue, RuntimeError> {
        self.api.kernel_read_substate(lock_handle)
    }

    fn kernel_write_substate(
        &mut self,
        lock_handle: SubstateHandle,
        value: IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        self.api.kernel_write_substate(lock_handle, value)
    }

    fn kernel_set_substate(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: SubstateKey,
        value: IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        self.api
            .kernel_set_substate(node_id, partition_num, substate_key, value)
    }

    fn kernel_remove_substate(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Result<Option<IndexedScryptoValue>, RuntimeError> {
        self.api
            .kernel_remove_substate(node_id, partition_num, substate_key)
    }

    fn kernel_scan_sorted_substates(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> Result<Vec<(SortedKey, IndexedScryptoValue)>, RuntimeError> {
        self.api
            .kernel_scan_sorted_substates(node_id, partition_num, count)
    }

    fn kernel_scan_keys<K: SubstateKeyContent + 'static>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> Result<Vec<SubstateKey>, RuntimeError> {
        self.api
            .kernel_scan_keys::<K>(node_id, partition_num, count)
    }

    fn kernel_drain_substates<K: SubstateKeyContent + 'static>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> Result<Vec<(SubstateKey, IndexedScryptoValue)>, RuntimeError> {
        self.api
            .kernel_drain_substates::<K>(node_id, partition_num, count)
    }
}

impl<'a, M: KernelCallbackObject + 'a, K: KernelApi<InjectCostingError<M>>>
    KernelInvokeApi<M::CallFrameData> for WrappedKernelApi<'a, M, K>
{
    fn kernel_invoke(
        &mut self,
        invocation: Box<KernelInvocation<M::CallFrameData>>,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        self.api.kernel_invoke(invocation)
    }
}

impl<'a, M: KernelCallbackObject, K: KernelApi<InjectCostingError<M>>> KernelInternalApi<M>
    for WrappedKernelApi<'a, M, K>
{
    fn kernel_get_system_state(&mut self) -> SystemState<'_, M> {
        let state = self.api.kernel_get_system_state();
        SystemState {
            system: &mut state.system.callback_object,
            caller_call_frame: state.caller_call_frame,
            current_call_frame: state.current_call_frame,
        }
    }

    fn kernel_get_current_depth(&self) -> usize {
        self.api.kernel_get_current_depth()
    }

    fn kernel_get_node_visibility(&self, node_id: &NodeId) -> NodeVisibility {
        self.api.kernel_get_node_visibility(node_id)
    }

    fn kernel_read_bucket(&mut self, bucket_id: &NodeId) -> Option<BucketSnapshot> {
        self.api.kernel_read_bucket(bucket_id)
    }

    fn kernel_read_proof(&mut self, proof_id: &NodeId) -> Option<ProofSnapshot> {
        self.api.kernel_read_proof(proof_id)
    }
}

impl<'a, M: KernelCallbackObject, K: KernelApi<InjectCostingError<M>>> KernelApi<M>
    for WrappedKernelApi<'a, M, K>
{
}

pub struct WrappedKernelInternalApi<
    'a,
    M: KernelCallbackObject + 'a,
    K: KernelInternalApi<InjectCostingError<M>>,
> {
    api: &'a mut K,
    phantom: PhantomData<M>,
}

impl<'a, M: KernelCallbackObject, K: KernelInternalApi<InjectCostingError<M>>> KernelInternalApi<M>
    for WrappedKernelInternalApi<'a, M, K>
{
    fn kernel_get_system_state(&mut self) -> SystemState<'_, M> {
        let state = self.api.kernel_get_system_state();
        SystemState {
            system: &mut state.system.callback_object,
            caller_call_frame: state.caller_call_frame,
            current_call_frame: state.current_call_frame,
        }
    }

    fn kernel_get_current_depth(&self) -> usize {
        self.api.kernel_get_current_depth()
    }

    fn kernel_get_node_visibility(&self, node_id: &NodeId) -> NodeVisibility {
        self.api.kernel_get_node_visibility(node_id)
    }

    fn kernel_read_bucket(&mut self, bucket_id: &NodeId) -> Option<BucketSnapshot> {
        self.api.kernel_read_bucket(bucket_id)
    }

    fn kernel_read_proof(&mut self, proof_id: &NodeId) -> Option<ProofSnapshot> {
        self.api.kernel_read_proof(proof_id)
    }
}
