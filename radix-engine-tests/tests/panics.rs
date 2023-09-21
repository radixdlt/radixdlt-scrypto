use radix_engine::errors::*;
use radix_engine::kernel::call_frame::*;
use radix_engine::kernel::kernel_api::*;
use radix_engine::system::actor::*;
#[cfg(not(feature = "alloc"))]
use radix_engine::system::system::SystemService;
use radix_engine::system::system_callback::*;
use radix_engine::system::system_modules::execution_trace::*;
use radix_engine::track::*;
use radix_engine::vm::wasm::*;
use radix_engine::vm::*;
use radix_engine_common::types::*;
use radix_engine_interface::prelude::*;
use radix_engine_store_interface::db_key_mapper::*;

#[cfg(feature = "std")]
#[test]
fn panics_at_the_system_layer_or_below_can_be_caught() {
    // Arrange
    let mut kernel = MockKernel;
    let mut system_service = SystemService {
        api: &mut kernel,
        phantom: Default::default(),
    };

    // Act
    let actor = system_service.actor_get_blueprint_id();

    // Assert
    assert!(matches!(
        actor,
        Err(RuntimeError::SystemError(SystemError::SystemPanic(..)))
    ))
}

macro_rules! panic1 {
    () => {
        panic!("This kernel only does one thing: panic.")
    };
}

pub struct MockKernel;

impl<'g> KernelApi<SystemConfig<Vm<'g, DefaultWasmEngine, NoExtension>>> for MockKernel {}

impl KernelNodeApi for MockKernel {
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

impl KernelSubstateApi<SystemLockData> for MockKernel {
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
        _: SystemLockData,
    ) -> Result<SubstateHandle, RuntimeError> {
        panic1!()
    }

    fn kernel_get_lock_data(&mut self, _: SubstateHandle) -> Result<SystemLockData, RuntimeError> {
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

    fn kernel_scan_keys<F: SubstateKeyContent + 'static>(
        &mut self,
        _: &NodeId,
        _: PartitionNumber,
        _: u32,
    ) -> Result<Vec<SubstateKey>, RuntimeError> {
        panic1!()
    }

    fn kernel_drain_substates<F: SubstateKeyContent + 'static>(
        &mut self,
        _: &NodeId,
        _: PartitionNumber,
        _: u32,
    ) -> Result<Vec<(SubstateKey, IndexedScryptoValue)>, RuntimeError> {
        panic1!()
    }
}

impl KernelInvokeApi<Actor> for MockKernel {
    fn kernel_invoke(
        &mut self,
        _: Box<KernelInvocation<Actor>>,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        panic1!()
    }
}

impl<'g> KernelInternalApi<SystemConfig<Vm<'g, DefaultWasmEngine, NoExtension>>> for MockKernel {
    fn kernel_get_system_state(
        &mut self,
    ) -> SystemState<'_, SystemConfig<Vm<'g, DefaultWasmEngine, NoExtension>>> {
        panic1!()
    }

    fn kernel_get_current_depth(&self) -> usize {
        panic1!()
    }

    fn kernel_get_node_visibility(&self, _: &NodeId) -> NodeVisibility {
        panic1!()
    }

    fn kernel_read_bucket(&mut self, _: &NodeId) -> Option<BucketSnapshot> {
        panic1!()
    }

    fn kernel_read_proof(&mut self, _: &NodeId) -> Option<ProofSnapshot> {
        panic1!()
    }
}
