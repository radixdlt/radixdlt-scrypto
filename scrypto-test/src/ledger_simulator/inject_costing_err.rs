use radix_common::prelude::*;
use radix_engine::errors::*;
use radix_engine::kernel::call_frame::{CallFrameInit, CallFrameMessage, NodeVisibility};
use radix_engine::kernel::kernel_api::*;
use radix_engine::kernel::kernel_callback_api::*;
use radix_engine::system::actor::Actor;
use radix_engine::system::system_callback::{System, SystemInit, SystemLockData};
use radix_engine::system::system_callback_api::SystemCallbackObject;
use radix_engine::system::system_modules::costing::{CostingError, FeeReserveError, OnApplyCost};
use radix_engine::track::*;
use radix_engine::transaction::TransactionReceipt;
use radix_engine::vm::wasm::DefaultWasmEngine;
use radix_engine::vm::Vm;
use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use radix_engine_interface::prelude::*;
use radix_substate_store_interface::db_key_mapper::{SpreadPrefixKeyMapper, SubstateKeyContent};
use radix_substate_store_interface::interface::SubstateDatabase;
use radix_transactions::model::ExecutableTransaction;

pub type InjectSystemCostingError<'a, E> = InjectCostingError<Vm<'a, DefaultWasmEngine, E>>;

#[derive(Clone)]
pub struct InjectCostingErrorInput<I> {
    pub system_input: I,
    pub error_after_count: u64,
}

pub struct InjectCostingError<K: SystemCallbackObject> {
    fail_after: Rc<RefCell<u64>>,
    system: System<K>,
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
        WrappedKernelApi { api: $api }
    };
}

macro_rules! wrapped_internal_api {
    ($api:ident) => {
        WrappedKernelInternalApi { api: $api }
    };
}

impl<K: SystemCallbackObject> KernelTransactionCallbackObject for InjectCostingError<K> {
    type Init = InjectCostingErrorInput<SystemInit<K::Init>>;
    type Executable = ExecutableTransaction;
    type ExecutionOutput = Vec<InstructionOutput>;
    type Receipt = TransactionReceipt;

    fn init<S: BootStore + CommitableSubstateStore>(
        store: &mut S,
        executable: &ExecutableTransaction,
        init_input: Self::Init,
    ) -> Result<(Self, Vec<CallFrameInit<Actor>>), Self::Receipt> {
        let (mut system, call_frame_inits) =
            System::<K>::init(store, executable, init_input.system_input)?;

        let fail_after = Rc::new(RefCell::new(init_input.error_after_count));
        system.modules.costing_mut().unwrap().on_apply_cost = OnApplyCost::ForceFailOnCount {
            fail_after: fail_after.clone(),
        };

        Ok((Self { fail_after, system }, call_frame_inits))
    }

    fn start<Y: KernelApi<CallbackObject = Self>>(
        api: &mut Y,
        executable: ExecutableTransaction,
    ) -> Result<Vec<InstructionOutput>, RuntimeError> {
        let mut api = wrapped_api!(api);
        System::start(&mut api, executable)
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

type InternalSystem<V> = System<V>;

impl<V: SystemCallbackObject> KernelCallbackObject for InjectCostingError<V> {
    type LockData = SystemLockData;
    type CallFrameData = Actor;

    fn on_pin_node<Y: KernelInternalApi<System = Self>>(
        node_id: &NodeId,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        InternalSystem::<V>::on_pin_node(node_id, &mut api)
    }

    fn on_create_node<Y: KernelInternalApi<System = Self>>(
        event: CreateNodeEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        InternalSystem::<V>::on_create_node(event, &mut api)
    }

    fn on_drop_node<Y: KernelInternalApi<System = Self>>(
        event: DropNodeEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        InternalSystem::<V>::on_drop_node(event, &mut api)
    }

    fn on_move_module<Y: KernelInternalApi<System = Self>>(
        event: MoveModuleEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        InternalSystem::<V>::on_move_module(event, &mut api)
    }

    fn on_open_substate<Y: KernelInternalApi<System = Self>>(
        event: OpenSubstateEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        InternalSystem::<V>::on_open_substate(event, &mut api)
    }

    fn on_close_substate<Y: KernelInternalApi<System = Self>>(
        event: CloseSubstateEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        InternalSystem::<V>::on_close_substate(event, &mut api)
    }

    fn on_read_substate<Y: KernelInternalApi<System = Self>>(
        event: ReadSubstateEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        InternalSystem::<V>::on_read_substate(event, &mut api)
    }

    fn on_write_substate<Y: KernelInternalApi<System = Self>>(
        event: WriteSubstateEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        InternalSystem::<V>::on_write_substate(event, &mut api)
    }

    fn on_set_substate<Y: KernelInternalApi<System = Self>>(
        event: SetSubstateEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        InternalSystem::<V>::on_set_substate(event, &mut api)
    }

    fn on_remove_substate<Y: KernelInternalApi<System = Self>>(
        event: RemoveSubstateEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        InternalSystem::<V>::on_remove_substate(event, &mut api)
    }

    fn on_scan_keys<Y: KernelInternalApi<System = Self>>(
        event: ScanKeysEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        InternalSystem::<V>::on_scan_keys(event, &mut api)
    }

    fn on_drain_substates<Y: KernelInternalApi<System = Self>>(
        event: DrainSubstatesEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        InternalSystem::<V>::on_drain_substates(event, &mut api)
    }

    fn on_scan_sorted_substates<Y: KernelInternalApi<System = Self>>(
        event: ScanSortedSubstatesEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        InternalSystem::<V>::on_scan_sorted_substates(event, &mut api)
    }

    fn before_invoke<Y: KernelApi<CallbackObject = Self>>(
        invocation: &KernelInvocation<Self::CallFrameData>,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        InternalSystem::<V>::before_invoke(invocation, &mut api)
    }

    fn on_execution_start<Y: KernelInternalApi<System = Self>>(
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        InternalSystem::<V>::on_execution_start(&mut api)
    }

    fn invoke_upstream<Y: KernelApi<CallbackObject = Self>>(
        args: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        InternalSystem::<V>::invoke_upstream(args, &mut api)
    }

    fn auto_drop<Y: KernelApi<CallbackObject = Self>>(
        nodes: Vec<NodeId>,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        InternalSystem::<V>::auto_drop(nodes, &mut api)
    }

    fn on_execution_finish<Y: KernelInternalApi<System = Self>>(
        message: &CallFrameMessage,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        InternalSystem::<V>::on_execution_finish(message, &mut api)
    }

    fn after_invoke<Y: KernelApi<CallbackObject = Self>>(
        output: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        InternalSystem::<V>::after_invoke(output, &mut api)
    }

    fn on_allocate_node_id<Y: KernelInternalApi<System = Self>>(
        entity_type: EntityType,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        InternalSystem::<V>::on_allocate_node_id(entity_type, &mut api)
    }

    fn on_mark_substate_as_transient<Y: KernelInternalApi<System = Self>>(
        node_id: &NodeId,
        partition_number: &PartitionNumber,
        substate_key: &SubstateKey,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        InternalSystem::<V>::on_mark_substate_as_transient(
            node_id,
            partition_number,
            substate_key,
            &mut api,
        )
    }

    fn on_substate_lock_fault<Y: KernelApi<CallbackObject = Self>>(
        node_id: NodeId,
        partition_num: PartitionNumber,
        offset: &SubstateKey,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        InternalSystem::<V>::on_substate_lock_fault(node_id, partition_num, offset, &mut api)
    }

    fn on_drop_node_mut<Y: KernelApi<CallbackObject = Self>>(
        node_id: &NodeId,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        InternalSystem::<V>::on_drop_node_mut(node_id, &mut api)
    }
}

pub struct WrappedKernelApi<
    'a,
    M: SystemCallbackObject + 'a,
    K: KernelApi<CallbackObject = InjectCostingError<M>>,
> {
    api: &'a mut K,
}

impl<'a, M: SystemCallbackObject, K: KernelApi<CallbackObject = InjectCostingError<M>>>
    KernelNodeApi for WrappedKernelApi<'a, M, K>
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

impl<'a, M: SystemCallbackObject, Y: KernelApi<CallbackObject = InjectCostingError<M>>>
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

impl<'a, M: SystemCallbackObject + 'a, K: KernelApi<CallbackObject = InjectCostingError<M>>>
    KernelInvokeApi<Actor> for WrappedKernelApi<'a, M, K>
{
    fn kernel_invoke(
        &mut self,
        invocation: Box<KernelInvocation<Actor>>,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        self.api.kernel_invoke(invocation)
    }
}

impl<'a, M: SystemCallbackObject, K: KernelApi<CallbackObject = InjectCostingError<M>>>
    KernelStackApi for WrappedKernelApi<'a, M, K>
{
    type CallFrameData = Actor;

    fn kernel_get_stack_id(&self) -> usize {
        self.api.kernel_get_stack_id()
    }

    fn kernel_switch_stack(&mut self, id: usize) -> Result<(), RuntimeError> {
        self.api.kernel_switch_stack(id)
    }

    fn kernel_set_call_frame_data(&mut self, data: Actor) -> Result<(), RuntimeError> {
        self.api.kernel_set_call_frame_data(data)
    }

    fn kernel_get_owned_nodes(&mut self) -> Result<Vec<NodeId>, RuntimeError> {
        self.api.kernel_get_owned_nodes()
    }
}

impl<'a, M: SystemCallbackObject, K: KernelApi<CallbackObject = InjectCostingError<M>>>
    KernelInternalApi for WrappedKernelApi<'a, M, K>
{
    type System = System<M>;

    fn kernel_get_system_state(&mut self) -> SystemState<'_, System<M>> {
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

    fn kernel_read_substate_uncosted(
        &self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Option<&IndexedScryptoValue> {
        self.api
            .kernel_read_substate_uncosted(node_id, partition_num, substate_key)
    }
}

impl<'a, M: SystemCallbackObject, K: KernelApi<CallbackObject = InjectCostingError<M>>> KernelApi
    for WrappedKernelApi<'a, M, K>
{
    type CallbackObject = System<M>;
}

pub struct WrappedKernelInternalApi<
    'a,
    M: SystemCallbackObject + 'a,
    K: KernelInternalApi<System = InjectCostingError<M>>,
> {
    api: &'a mut K,
}

impl<'a, M: SystemCallbackObject, K: KernelInternalApi<System = InjectCostingError<M>>>
    KernelInternalApi for WrappedKernelInternalApi<'a, M, K>
{
    type System = System<M>;

    fn kernel_get_system_state(&mut self) -> SystemState<'_, System<M>> {
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

    fn kernel_read_substate_uncosted(
        &self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Option<&IndexedScryptoValue> {
        self.api
            .kernel_read_substate_uncosted(node_id, partition_num, substate_key)
    }
}
