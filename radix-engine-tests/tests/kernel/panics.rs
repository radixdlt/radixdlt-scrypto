use radix_common::prelude::*;
use radix_engine::errors::*;
use radix_engine::kernel::call_frame::*;
use radix_engine::kernel::kernel_api::*;
#[cfg(not(feature = "alloc"))]
use radix_engine::system::system::SystemService;
use radix_engine::system::system_callback::*;
use radix_engine::track::*;
use radix_engine::vm::wasm::*;
use radix_engine::vm::*;
use radix_engine_interface::prelude::*;
use radix_substate_store_interface::db_key_mapper::*;
use scrypto_test::prelude::*;

#[cfg(feature = "std")]
#[test]
fn panics_at_the_system_layer_or_below_can_be_caught() {
    // Arrange
    let mut kernel = MockKernel(PhantomData::<System<Vm<DefaultWasmEngine, NoExtension>>>);
    let mut system_service = SystemService::new(&mut kernel);

    // Act
    let actor = system_service.actor_get_blueprint_id();

    // Assert
    assert_matches!(
        actor,
        Err(RuntimeError::SystemError(SystemError::SystemPanic(..)))
    )
}

macro_rules! panic1 {
    () => {
        panic!("This kernel only does one thing: panic.")
    };
}

pub struct MockKernel<E: KernelTransactionExecutor>(PhantomData<E>);

impl<E: KernelTransactionExecutor> KernelApi for MockKernel<E> {
    type CallbackObject = E;
}

impl<E: KernelTransactionExecutor> KernelStackApi for MockKernel<E> {
    type CallFrameData = E::CallFrameData;

    fn kernel_get_stack_id(&mut self) -> Result<usize, RuntimeError> {
        panic1!()
    }

    fn kernel_switch_stack(&mut self, _id: usize) -> Result<(), RuntimeError> {
        panic1!()
    }

    fn kernel_send_to_stack(
        &mut self,
        _id: usize,
        _value: &IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        panic1!()
    }

    fn kernel_set_call_frame_data(&mut self, _data: E::CallFrameData) -> Result<(), RuntimeError> {
        panic1!()
    }

    fn kernel_get_owned_nodes(&mut self) -> Result<Vec<NodeId>, RuntimeError> {
        panic1!()
    }
}

impl<E: KernelTransactionExecutor> KernelNodeApi for MockKernel<E> {
    fn kernel_pin_node(&mut self, _: NodeId) -> Result<(), RuntimeError> {
        panic1!()
    }

    fn kernel_allocate_node_id(&mut self, _: EntityType) -> Result<NodeId, RuntimeError> {
        panic1!()
    }

    fn kernel_create_node(&mut self, _: NodeId, _: NodeSubstates) -> Result<(), RuntimeError> {
        panic1!()
    }

    fn kernel_create_node_from(
        &mut self,
        _: NodeId,
        _: BTreeMap<PartitionNumber, (NodeId, PartitionNumber)>,
    ) -> Result<(), RuntimeError> {
        panic1!()
    }

    fn kernel_drop_node(&mut self, _: &NodeId) -> Result<DroppedNode, RuntimeError> {
        panic1!()
    }
}

impl<E: KernelTransactionExecutor> KernelSubstateApi<E::LockData> for MockKernel<E> {
    fn kernel_mark_substate_as_transient(
        &mut self,
        _: NodeId,
        _: PartitionNumber,
        _: SubstateKey,
    ) -> Result<(), RuntimeError> {
        panic1!()
    }

    fn kernel_open_substate_with_default<F: FnOnce() -> IndexedScryptoValue>(
        &mut self,
        _: &NodeId,
        _: PartitionNumber,
        _: &SubstateKey,
        _: LockFlags,
        _: Option<F>,
        _: E::LockData,
    ) -> Result<SubstateHandle, RuntimeError> {
        panic1!()
    }

    fn kernel_get_lock_data(&mut self, _: SubstateHandle) -> Result<E::LockData, RuntimeError> {
        panic1!()
    }

    fn kernel_close_substate(&mut self, _: SubstateHandle) -> Result<(), RuntimeError> {
        panic1!()
    }

    fn kernel_read_substate(
        &mut self,
        _: SubstateHandle,
    ) -> Result<&IndexedScryptoValue, RuntimeError> {
        panic1!()
    }

    fn kernel_write_substate(
        &mut self,
        _: SubstateHandle,
        _: IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        panic1!()
    }

    fn kernel_set_substate(
        &mut self,
        _: &NodeId,
        _: PartitionNumber,
        _: SubstateKey,
        _: IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        panic1!()
    }

    fn kernel_remove_substate(
        &mut self,
        _: &NodeId,
        _: PartitionNumber,
        _: &SubstateKey,
    ) -> Result<Option<IndexedScryptoValue>, RuntimeError> {
        panic1!()
    }

    fn kernel_scan_sorted_substates(
        &mut self,
        _: &NodeId,
        _: PartitionNumber,
        _: u32,
    ) -> Result<Vec<(SortedKey, IndexedScryptoValue)>, RuntimeError> {
        panic1!()
    }

    fn kernel_scan_keys<F: SubstateKeyContent>(
        &mut self,
        _: &NodeId,
        _: PartitionNumber,
        _: u32,
    ) -> Result<Vec<SubstateKey>, RuntimeError> {
        panic1!()
    }

    fn kernel_drain_substates<F: SubstateKeyContent>(
        &mut self,
        _: &NodeId,
        _: PartitionNumber,
        _: u32,
    ) -> Result<Vec<(SubstateKey, IndexedScryptoValue)>, RuntimeError> {
        panic1!()
    }
}

impl<E: KernelTransactionExecutor> KernelInvokeApi<E::CallFrameData> for MockKernel<E> {
    fn kernel_invoke(
        &mut self,
        _: Box<KernelInvocation<E::CallFrameData>>,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        panic1!()
    }
}

impl<E: KernelTransactionExecutor> KernelInternalApi for MockKernel<E> {
    type System = E;

    fn kernel_get_system_state(&mut self) -> SystemState<'_, Self::System> {
        panic1!()
    }

    fn kernel_get_current_stack_depth_uncosted(&self) -> usize {
        panic1!()
    }

    fn kernel_get_current_stack_id_uncosted(&self) -> usize {
        panic1!()
    }

    fn kernel_get_node_visibility_uncosted(&self, _: &NodeId) -> NodeVisibility {
        panic1!()
    }

    fn kernel_read_substate_uncosted(
        &self,
        _node_id: &NodeId,
        _partition_num: PartitionNumber,
        _substate_key: &SubstateKey,
    ) -> Option<&IndexedScryptoValue> {
        panic1!()
    }
}
