use radix_common::prelude::*;
use radix_engine::errors::{RejectionReason, TransactionExecutionError};
use radix_engine::errors::{RuntimeError, SystemModuleError};
use radix_engine::kernel::call_frame::{CallFrameMessage, NodeVisibility};
use radix_engine::kernel::kernel_api::{
    DroppedNode, KernelApi, KernelInternalApi, KernelInvocation, KernelInvokeApi, KernelNodeApi,
    KernelStackApi, KernelSubstateApi, SystemState,
};
use radix_engine::kernel::kernel_callback_api::{
    CloseSubstateEvent, CreateNodeEvent, DrainSubstatesEvent, DropNodeEvent, KernelCallbackObject,
    KernelStacksInit, KernelTransactionCallbackObject, MoveModuleEvent, OpenSubstateEvent,
    ReadSubstateEvent, RemoveSubstateEvent, ScanKeysEvent, ScanSortedSubstatesEvent,
    SetSubstateEvent, WriteSubstateEvent,
};
use radix_engine::system::actor::Actor;
use radix_engine::system::system_callback::{System, SystemInit, SystemLockData};
use radix_engine::system::system_callback_api::SystemCallbackObject;
use radix_engine::system::system_modules::costing::{CostingError, FeeReserveError, OnApplyCost};
use radix_engine::system::system_modules::execution_trace::{BucketSnapshot, ProofSnapshot};
use radix_engine::track::{
    BootStore, CommitableSubstateStore, NodeSubstates, StoreCommitInfo, Track,
};
use radix_engine::transaction::TransactionReceipt;
use radix_engine::vm::wasm::DefaultWasmEngine;
use radix_engine::vm::Vm;
use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use radix_engine_interface::prelude::*;
use radix_substate_store_interface::db_key_mapper::{SpreadPrefixKeyMapper, SubstateKeyContent};
use radix_substate_store_interface::interface::SubstateDatabase;
use radix_transactions::model::Executable;

pub type InjectSystemCostingError<'a, E> = InjectCostingError<Vm<'a, DefaultWasmEngine, E>>;

#[derive(Clone)]
pub struct InjectCostingErrorInput<I> {
    pub system_input: I,
    pub error_after_count: u64,
}

pub struct InjectCostingError<K: SystemCallbackObject> {
    fail_after: Rc<RefCell<u64>>,
    system: System<K, Executable>,
}

impl<K: SystemCallbackObject> InjectCostingError<K> {
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

impl<K: SystemCallbackObject> KernelTransactionCallbackObject for InjectCostingError<K> {
    type Init = InjectCostingErrorInput<SystemInit<K::Init>>;
    type Executable = Executable;
    type ExecutionOutput = Vec<InstructionOutput>;
    type Receipt = TransactionReceipt;

    fn init<S: BootStore + CommitableSubstateStore>(
        store: &mut S,
        executable: Executable,
        init_input: Self::Init,
    ) -> Result<(Self, KernelStacksInit<Actor>), RejectionReason> {
        let (mut system, kernel_stacks_init) =
            System::<K, _>::init(store, executable, init_input.system_input)?;

        let fail_after = Rc::new(RefCell::new(init_input.error_after_count));
        system.modules.costing_mut().unwrap().on_apply_cost = OnApplyCost::ForceFailOnCount {
            fail_after: fail_after.clone(),
        };

        Ok((Self { fail_after, system }, kernel_stacks_init))
    }

    fn start<Y: KernelApi<Self>>(api: &mut Y) -> Result<Vec<InstructionOutput>, RuntimeError> {
        let mut api = wrapped_api!(api);
        System::start(&mut api)
    }

    fn finish(&mut self, store_commit_info: StoreCommitInfo) -> Result<(), RuntimeError> {
        self.maybe_err()?;
        self.system.finish(store_commit_info)
    }

    fn create_receipt<S: SubstateDatabase>(
        self,
        track: Track<S, SpreadPrefixKeyMapper>,
        result: Result<Vec<InstructionOutput>, TransactionExecutionError>,
    ) -> TransactionReceipt {
        self.system.create_receipt(track, result)
    }
}

impl<K: SystemCallbackObject> KernelCallbackObject for InjectCostingError<K> {
    type LockData = SystemLockData;
    type CallFrameData = Actor;

    fn on_pin_node(&mut self, node_id: &NodeId) -> Result<(), RuntimeError> {
        self.maybe_err()?;
        self.system.on_pin_node(node_id)
    }

    fn on_create_node<Y: KernelInternalApi<Self>>(
        api: &mut Y,
        event: CreateNodeEvent,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        System::<_, Executable>::on_create_node(&mut api, event)
    }

    fn on_drop_node<Y: KernelInternalApi<Self>>(
        api: &mut Y,
        event: DropNodeEvent,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        System::<_, Executable>::on_drop_node(&mut api, event)
    }

    fn on_move_module<Y: KernelInternalApi<Self>>(
        api: &mut Y,
        event: MoveModuleEvent,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        System::<_, Executable>::on_move_module(&mut api, event)
    }

    fn on_open_substate<Y: KernelInternalApi<Self>>(
        api: &mut Y,
        event: OpenSubstateEvent,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        System::<_, Executable>::on_open_substate(&mut api, event)
    }

    fn on_close_substate<Y: KernelInternalApi<Self>>(
        api: &mut Y,
        event: CloseSubstateEvent,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        System::<_, Executable>::on_close_substate(&mut api, event)
    }

    fn on_read_substate<Y: KernelInternalApi<Self>>(
        api: &mut Y,
        event: ReadSubstateEvent,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        System::<_, Executable>::on_read_substate(&mut api, event)
    }

    fn on_write_substate<Y: KernelInternalApi<Self>>(
        api: &mut Y,
        event: WriteSubstateEvent,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        System::<_, Executable>::on_write_substate(&mut api, event)
    }

    fn on_set_substate(&mut self, event: SetSubstateEvent) -> Result<(), RuntimeError> {
        self.maybe_err()?;
        self.system.on_set_substate(event)
    }

    fn on_remove_substate(&mut self, event: RemoveSubstateEvent) -> Result<(), RuntimeError> {
        self.maybe_err()?;
        self.system.on_remove_substate(event)
    }

    fn on_scan_keys(&mut self, event: ScanKeysEvent) -> Result<(), RuntimeError> {
        self.maybe_err()?;
        self.system.on_scan_keys(event)
    }

    fn on_drain_substates(&mut self, event: DrainSubstatesEvent) -> Result<(), RuntimeError> {
        self.maybe_err()?;
        self.system.on_drain_substates(event)
    }

    fn on_scan_sorted_substates(
        &mut self,
        event: ScanSortedSubstatesEvent,
    ) -> Result<(), RuntimeError> {
        self.maybe_err()?;
        self.system.on_scan_sorted_substates(event)
    }

    fn before_invoke<Y: KernelApi<Self>>(
        invocation: &KernelInvocation<Self::CallFrameData>,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        System::<_, Executable>::before_invoke(invocation, &mut api)
    }

    fn on_execution_start<Y: KernelApi<Self>>(api: &mut Y) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        System::<_, Executable>::on_execution_start(&mut api)
    }

    fn invoke_upstream<Y: KernelApi<Self>>(
        args: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        System::<_, Executable>::invoke_upstream(args, &mut api)
    }

    fn auto_drop<Y: KernelApi<Self>>(nodes: Vec<NodeId>, api: &mut Y) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        System::<_, Executable>::auto_drop(nodes, &mut api)
    }

    fn on_execution_finish<Y: KernelApi<Self>>(
        message: &CallFrameMessage,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        System::<_, Executable>::on_execution_finish(message, &mut api)
    }

    fn after_invoke<Y: KernelApi<Self>>(
        output: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        System::<_, Executable>::after_invoke(output, &mut api)
    }

    fn on_allocate_node_id<Y: KernelApi<Self>>(
        entity_type: EntityType,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        System::<_, Executable>::on_allocate_node_id(entity_type, &mut api)
    }

    fn on_mark_substate_as_transient(
        &mut self,
        node_id: &NodeId,
        partition_number: &PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Result<(), RuntimeError> {
        self.maybe_err()?;
        self.system
            .on_mark_substate_as_transient(node_id, partition_number, substate_key)
    }

    fn on_substate_lock_fault<Y: KernelApi<Self>>(
        node_id: NodeId,
        partition_num: PartitionNumber,
        offset: &SubstateKey,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        System::<_, Executable>::on_substate_lock_fault(node_id, partition_num, offset, &mut api)
    }

    fn on_drop_node_mut<Y: KernelApi<Self>>(
        node_id: &NodeId,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        System::<_, Executable>::on_drop_node_mut(node_id, &mut api)
    }
}

pub struct WrappedKernelApi<'a, M: SystemCallbackObject + 'a, K: KernelApi<InjectCostingError<M>>> {
    api: &'a mut K,
    phantom: PhantomData<M>,
}

impl<'a, M: SystemCallbackObject, K: KernelApi<InjectCostingError<M>>> KernelStackApi<Actor>
    for WrappedKernelApi<'a, M, K>
{
    fn kernel_switch_stack(&mut self, to_stack_id: Hash) -> Result<(), RuntimeError> {
        self.api.kernel_switch_stack(to_stack_id)
    }

    fn kernel_send_to_stack(
        &mut self,
        to_stack_id: Hash,
        value: IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        self.api.kernel_send_to_stack(to_stack_id, value)
    }

    fn kernel_free_and_switch_stack(&mut self, to_stack_id: Hash) -> Result<(), RuntimeError> {
        self.api.kernel_free_and_switch_stack(to_stack_id)
    }

    fn kernel_set_call_frame_data(&mut self, data: Actor) -> Result<(), RuntimeError> {
        self.api.kernel_set_call_frame_data(data)
    }
}

impl<'a, M: SystemCallbackObject, K: KernelApi<InjectCostingError<M>>> KernelNodeApi
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

impl<'a, M: SystemCallbackObject, Y: KernelApi<InjectCostingError<M>>>
    KernelSubstateApi<SystemLockData> for WrappedKernelApi<'a, M, Y>
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
        lock_data: SystemLockData,
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
    ) -> Result<SystemLockData, RuntimeError> {
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

impl<'a, M: SystemCallbackObject + 'a, K: KernelApi<InjectCostingError<M>>> KernelInvokeApi<Actor>
    for WrappedKernelApi<'a, M, K>
{
    fn kernel_invoke(
        &mut self,
        invocation: Box<KernelInvocation<Actor>>,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        self.api.kernel_invoke(invocation)
    }
}

impl<'a, M: SystemCallbackObject, K: KernelApi<InjectCostingError<M>>>
    KernelInternalApi<System<M, Executable>> for WrappedKernelApi<'a, M, K>
{
    fn kernel_get_system_state(&mut self) -> SystemState<'_, System<M, Executable>> {
        let state = self.api.kernel_get_system_state();
        SystemState {
            system: &mut state.system.system,
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

impl<'a, M: SystemCallbackObject, K: KernelApi<InjectCostingError<M>>>
    KernelApi<System<M, Executable>> for WrappedKernelApi<'a, M, K>
{
}

pub struct WrappedKernelInternalApi<
    'a,
    M: SystemCallbackObject + 'a,
    K: KernelInternalApi<InjectCostingError<M>>,
> {
    api: &'a mut K,
    phantom: PhantomData<M>,
}

impl<'a, M: SystemCallbackObject, K: KernelInternalApi<InjectCostingError<M>>>
    KernelInternalApi<System<M, Executable>> for WrappedKernelInternalApi<'a, M, K>
{
    fn kernel_get_system_state(&mut self) -> SystemState<'_, System<M, Executable>> {
        let state = self.api.kernel_get_system_state();
        SystemState {
            system: &mut state.system.system,
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
