use radix_engine::init::InitializationParameters;

use crate::prelude::*;

#[derive(Clone)]
pub struct InjectCostingErrorInit<I> {
    pub system_input: I,
    pub error_after_count: u64,
}

impl<I: InitializationParameters<For: KernelTransactionExecutor>> InitializationParameters
    for InjectCostingErrorInit<I>
{
    type For = InjectCostingError<I::For>;
}

pub struct InjectCostingError<Y: KernelTransactionExecutor> {
    fail_after: Rc<RefCell<u64>>,
    wrapped: Y,
}

impl<Y: KernelTransactionExecutor> InjectCostingError<Y> {
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

impl<
        E: KernelTransactionExecutor<
                Executable = ExecutableTransaction,
                ExecutionOutput = Vec<InstructionOutput>,
                Receipt = TransactionReceipt,
            > + HasModules,
    > KernelTransactionExecutor for InjectCostingError<E>
{
    type Init = InjectCostingErrorInit<E::Init>;
    type Executable = ExecutableTransaction;
    type ExecutionOutput = Vec<InstructionOutput>;
    type Receipt = TransactionReceipt;

    fn init(
        store: &mut impl CommitableSubstateStore,
        executable: &ExecutableTransaction,
        init_input: Self::Init,
        always_visible_global_nodes: &'static IndexSet<NodeId>,
    ) -> Result<(Self, Vec<CallFrameInit<Self::CallFrameData>>), Self::Receipt> {
        let (mut system, call_frame_inits) = E::init(
            store,
            executable,
            init_input.system_input,
            always_visible_global_nodes,
        )?;

        let fail_after = Rc::new(RefCell::new(init_input.error_after_count));
        system.modules_mut().costing_mut().unwrap().on_apply_cost = OnApplyCost::ForceFailOnCount {
            fail_after: fail_after.clone(),
        };

        Ok((
            Self {
                fail_after,
                wrapped: system,
            },
            call_frame_inits,
        ))
    }

    fn execute<Y: KernelApi<CallbackObject = Self>>(
        api: &mut Y,
        executable: &ExecutableTransaction,
    ) -> Result<Vec<InstructionOutput>, RuntimeError> {
        let mut api = wrapped_api!(api);
        E::execute(&mut api, executable)
    }

    fn finalize(
        &mut self,
        executable: &ExecutableTransaction,
        store_commit_info: StoreCommitInfo,
    ) -> Result<(), RuntimeError> {
        self.maybe_err()?;
        self.wrapped.finalize(executable, store_commit_info)
    }

    fn create_receipt<S: SubstateDatabase>(
        self,
        track: Track<S>,
        result: Result<Vec<InstructionOutput>, TransactionExecutionError>,
    ) -> TransactionReceipt {
        self.wrapped.create_receipt(track, result)
    }
}

impl<E: KernelTransactionExecutor> KernelCallbackObject for InjectCostingError<E> {
    type LockData = E::LockData;
    type CallFrameData = E::CallFrameData;

    fn on_pin_node<Y: KernelInternalApi<System = Self>>(
        node_id: &NodeId,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        E::on_pin_node(node_id, &mut api)
    }

    fn on_create_node<Y: KernelInternalApi<System = Self>>(
        event: CreateNodeEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        E::on_create_node(event, &mut api)
    }

    fn on_drop_node<Y: KernelInternalApi<System = Self>>(
        event: DropNodeEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        E::on_drop_node(event, &mut api)
    }

    fn on_move_module<Y: KernelInternalApi<System = Self>>(
        event: MoveModuleEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        E::on_move_module(event, &mut api)
    }

    fn on_open_substate<Y: KernelInternalApi<System = Self>>(
        event: OpenSubstateEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        E::on_open_substate(event, &mut api)
    }

    fn on_close_substate<Y: KernelInternalApi<System = Self>>(
        event: CloseSubstateEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        E::on_close_substate(event, &mut api)
    }

    fn on_read_substate<Y: KernelInternalApi<System = Self>>(
        event: ReadSubstateEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        E::on_read_substate(event, &mut api)
    }

    fn on_write_substate<Y: KernelInternalApi<System = Self>>(
        event: WriteSubstateEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        E::on_write_substate(event, &mut api)
    }

    fn on_set_substate<Y: KernelInternalApi<System = Self>>(
        event: SetSubstateEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        E::on_set_substate(event, &mut api)
    }

    fn on_remove_substate<Y: KernelInternalApi<System = Self>>(
        event: RemoveSubstateEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        E::on_remove_substate(event, &mut api)
    }

    fn on_scan_keys<Y: KernelInternalApi<System = Self>>(
        event: ScanKeysEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        E::on_scan_keys(event, &mut api)
    }

    fn on_drain_substates<Y: KernelInternalApi<System = Self>>(
        event: DrainSubstatesEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        E::on_drain_substates(event, &mut api)
    }

    fn on_scan_sorted_substates<Y: KernelInternalApi<System = Self>>(
        event: ScanSortedSubstatesEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        E::on_scan_sorted_substates(event, &mut api)
    }

    fn before_invoke<Y: KernelApi<CallbackObject = Self>>(
        invocation: &KernelInvocation<Self::CallFrameData>,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        E::before_invoke(invocation, &mut api)
    }

    fn on_execution_start<Y: KernelInternalApi<System = Self>>(
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        E::on_execution_start(&mut api)
    }

    fn invoke_upstream<Y: KernelApi<CallbackObject = Self>>(
        args: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        E::invoke_upstream(args, &mut api)
    }

    fn auto_drop<Y: KernelApi<CallbackObject = Self>>(
        nodes: Vec<NodeId>,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        E::auto_drop(nodes, &mut api)
    }

    fn on_execution_finish<Y: KernelInternalApi<System = Self>>(
        message: &CallFrameMessage,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        E::on_execution_finish(message, &mut api)
    }

    fn after_invoke<Y: KernelApi<CallbackObject = Self>>(
        output: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        E::after_invoke(output, &mut api)
    }

    fn on_allocate_node_id<Y: KernelInternalApi<System = Self>>(
        entity_type: EntityType,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        E::on_allocate_node_id(entity_type, &mut api)
    }

    fn on_mark_substate_as_transient<Y: KernelInternalApi<System = Self>>(
        node_id: &NodeId,
        partition_number: &PartitionNumber,
        substate_key: &SubstateKey,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        E::on_mark_substate_as_transient(node_id, partition_number, substate_key, &mut api)
    }

    fn on_substate_lock_fault<Y: KernelApi<CallbackObject = Self>>(
        node_id: NodeId,
        partition_num: PartitionNumber,
        offset: &SubstateKey,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        E::on_substate_lock_fault(node_id, partition_num, offset, &mut api)
    }

    fn on_drop_node_mut<Y: KernelApi<CallbackObject = Self>>(
        node_id: &NodeId,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_api!(api);
        E::on_drop_node_mut(node_id, &mut api)
    }

    fn on_get_stack_id<Y: KernelInternalApi<System = Self>>(
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        E::on_get_stack_id(&mut api)
    }

    fn on_switch_stack<Y: KernelInternalApi<System = Self>>(
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        E::on_switch_stack(&mut api)
    }

    fn on_send_to_stack<Y: KernelInternalApi<System = Self>>(
        value: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        E::on_send_to_stack(value, &mut api)
    }

    fn on_set_call_frame_data<Y: KernelInternalApi<System = Self>>(
        data: &Self::CallFrameData,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        E::on_set_call_frame_data(data, &mut api)
    }

    fn on_get_owned_nodes<Y: KernelInternalApi<System = Self>>(
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system_state().system.maybe_err()?;
        let mut api = wrapped_internal_api!(api);
        E::on_get_owned_nodes(&mut api)
    }
}

pub struct WrappedKernelApi<
    'a,
    E: KernelTransactionExecutor + 'a,
    K: KernelApi<CallbackObject = InjectCostingError<E>>,
> {
    api: &'a mut K,
}

impl<
        'a,
        E: KernelTransactionExecutor + 'a,
        K: KernelApi<CallbackObject = InjectCostingError<E>>,
    > KernelNodeApi for WrappedKernelApi<'a, E, K>
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

impl<
        'a,
        E: KernelTransactionExecutor + 'a,
        Y: KernelApi<CallbackObject = InjectCostingError<E>>,
    > KernelSubstateApi<E::LockData> for WrappedKernelApi<'a, E, Y>
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
        lock_data: E::LockData,
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
    ) -> Result<E::LockData, RuntimeError> {
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

    fn kernel_scan_keys<K: SubstateKeyContent>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> Result<Vec<SubstateKey>, RuntimeError> {
        self.api
            .kernel_scan_keys::<K>(node_id, partition_num, count)
    }

    fn kernel_drain_substates<K: SubstateKeyContent>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> Result<Vec<(SubstateKey, IndexedScryptoValue)>, RuntimeError> {
        self.api
            .kernel_drain_substates::<K>(node_id, partition_num, count)
    }
}

impl<
        'a,
        E: KernelTransactionExecutor + 'a,
        K: KernelApi<CallbackObject = InjectCostingError<E>>,
    > KernelInvokeApi<E::CallFrameData> for WrappedKernelApi<'a, E, K>
{
    fn kernel_invoke(
        &mut self,
        invocation: Box<KernelInvocation<E::CallFrameData>>,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        self.api.kernel_invoke(invocation)
    }
}

impl<
        'a,
        E: KernelTransactionExecutor + 'a,
        K: KernelApi<CallbackObject = InjectCostingError<E>>,
    > KernelStackApi for WrappedKernelApi<'a, E, K>
{
    type CallFrameData = E::CallFrameData;

    fn kernel_get_stack_id(&mut self) -> Result<usize, RuntimeError> {
        self.api.kernel_get_stack_id()
    }

    fn kernel_switch_stack(&mut self, id: usize) -> Result<(), RuntimeError> {
        self.api.kernel_switch_stack(id)
    }

    fn kernel_send_to_stack(
        &mut self,
        id: usize,
        value: &IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        self.api.kernel_send_to_stack(id, value)
    }

    fn kernel_set_call_frame_data(&mut self, data: E::CallFrameData) -> Result<(), RuntimeError> {
        self.api.kernel_set_call_frame_data(data)
    }

    fn kernel_get_owned_nodes(&mut self) -> Result<Vec<NodeId>, RuntimeError> {
        self.api.kernel_get_owned_nodes()
    }
}

impl<
        'a,
        E: KernelTransactionExecutor + 'a,
        K: KernelApi<CallbackObject = InjectCostingError<E>>,
    > KernelInternalApi for WrappedKernelApi<'a, E, K>
{
    type System = E;

    fn kernel_get_system_state(&mut self) -> SystemState<'_, E> {
        let state = self.api.kernel_get_system_state();
        SystemState {
            system: &mut state.system.wrapped,
            caller_call_frame: state.caller_call_frame,
            current_call_frame: state.current_call_frame,
        }
    }

    fn kernel_get_current_stack_depth_uncosted(&self) -> usize {
        self.api.kernel_get_current_stack_depth_uncosted()
    }

    fn kernel_get_current_stack_id_uncosted(&self) -> usize {
        self.api.kernel_get_current_stack_id_uncosted()
    }

    fn kernel_get_node_visibility_uncosted(&self, node_id: &NodeId) -> NodeVisibility {
        self.api.kernel_get_node_visibility_uncosted(node_id)
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

impl<
        'a,
        E: KernelTransactionExecutor + 'a,
        K: KernelApi<CallbackObject = InjectCostingError<E>>,
    > KernelApi for WrappedKernelApi<'a, E, K>
{
    type CallbackObject = E;
}

pub struct WrappedKernelInternalApi<
    'a,
    E: KernelTransactionExecutor + 'a,
    K: KernelInternalApi<System = InjectCostingError<E>>,
> {
    api: &'a mut K,
}

impl<
        'a,
        E: KernelTransactionExecutor + 'a,
        K: KernelInternalApi<System = InjectCostingError<E>>,
    > KernelInternalApi for WrappedKernelInternalApi<'a, E, K>
{
    type System = E;

    fn kernel_get_system_state(&mut self) -> SystemState<'_, E> {
        let state = self.api.kernel_get_system_state();
        SystemState {
            system: &mut state.system.wrapped,
            caller_call_frame: state.caller_call_frame,
            current_call_frame: state.current_call_frame,
        }
    }

    fn kernel_get_current_stack_depth_uncosted(&self) -> usize {
        self.api.kernel_get_current_stack_depth_uncosted()
    }

    fn kernel_get_current_stack_id_uncosted(&self) -> usize {
        self.api.kernel_get_current_stack_id_uncosted()
    }

    fn kernel_get_node_visibility_uncosted(&self, node_id: &NodeId) -> NodeVisibility {
        self.api.kernel_get_node_visibility_uncosted(node_id)
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
