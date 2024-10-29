use radix_common::prelude::*;
use radix_engine::errors::*;
use radix_engine::kernel::call_frame::*;
use radix_engine::kernel::id_allocator::IdAllocator;
use radix_engine::kernel::kernel::Kernel;
use radix_engine::kernel::kernel_api::*;
use radix_engine::kernel::kernel_callback_api::*;
use radix_engine::track::*;
use radix_engine_interface::prelude::*;
use scrypto_test::prelude::*;

#[derive(Default)]
struct TestCallFrameData;

impl CallFrameReferences for TestCallFrameData {
    fn global_references(&self) -> Vec<NodeId> {
        Default::default()
    }

    fn direct_access_references(&self) -> Vec<NodeId> {
        Default::default()
    }

    fn stable_transient_references(&self) -> Vec<NodeId> {
        Default::default()
    }

    fn len(&self) -> usize {
        0usize
    }
}

struct TestCallbackObject;
impl KernelCallbackObject for TestCallbackObject {
    type LockData = ();
    type CallFrameData = TestCallFrameData;

    fn on_pin_node<Y: KernelInternalApi<System = Self>>(
        _node_id: &NodeId,
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_create_node<Y: KernelInternalApi<System = Self>>(
        _event: CreateNodeEvent,
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_drop_node<Y: KernelInternalApi<System = Self>>(
        _event: DropNodeEvent,
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_move_module<Y: KernelInternalApi<System = Self>>(
        _event: MoveModuleEvent,
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_open_substate<Y: KernelInternalApi<System = Self>>(
        _event: OpenSubstateEvent,
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_close_substate<Y: KernelInternalApi<System = Self>>(
        _event: CloseSubstateEvent,
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_read_substate<Y: KernelInternalApi<System = Self>>(
        _event: ReadSubstateEvent,
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_write_substate<Y: KernelInternalApi<System = Self>>(
        _event: WriteSubstateEvent,
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_set_substate<Y: KernelInternalApi<System = Self>>(
        _event: SetSubstateEvent,
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_remove_substate<Y: KernelInternalApi<System = Self>>(
        _event: RemoveSubstateEvent,
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_scan_keys<Y: KernelInternalApi<System = Self>>(
        _event: ScanKeysEvent,
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_drain_substates<Y: KernelInternalApi<System = Self>>(
        _event: DrainSubstatesEvent,
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_scan_sorted_substates<Y: KernelInternalApi<System = Self>>(
        _event: ScanSortedSubstatesEvent,
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn before_invoke<Y: KernelApi<CallbackObject = Self>>(
        _invocation: &KernelInvocation<Self::CallFrameData>,
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn after_invoke<Y: KernelApi<CallbackObject = Self>>(
        _output: &IndexedScryptoValue,
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_execution_start<Y: KernelInternalApi<System = Self>>(
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_execution_finish<Y: KernelInternalApi<System = Self>>(
        _message: &CallFrameMessage,
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_allocate_node_id<Y: KernelInternalApi<System = Self>>(
        _entity_type: EntityType,
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn invoke_upstream<Y: KernelApi<CallbackObject = Self>>(
        args: &IndexedScryptoValue,
        _api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        Ok(args.clone())
    }

    fn auto_drop<Y: KernelApi<CallbackObject = Self>>(
        _nodes: Vec<NodeId>,
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_mark_substate_as_transient<Y: KernelInternalApi<System = Self>>(
        _node_id: &NodeId,
        _partition_number: &PartitionNumber,
        _substate_key: &SubstateKey,
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_substate_lock_fault<Y: KernelApi<CallbackObject = Self>>(
        _node_id: NodeId,
        _partition_num: PartitionNumber,
        _offset: &SubstateKey,
        _api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        Ok(false)
    }

    fn on_drop_node_mut<Y: KernelApi<CallbackObject = Self>>(
        _node_id: &NodeId,
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_get_stack_id<Y: KernelInternalApi<System = Self>>(
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_switch_stack<Y: KernelInternalApi<System = Self>>(
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_send_to_stack<Y: KernelInternalApi<System = Self>>(
        _value: &IndexedScryptoValue,
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_set_call_frame_data<Y: KernelInternalApi<System = Self>>(
        _data: &Self::CallFrameData,
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_get_owned_nodes<Y: KernelInternalApi<System = Self>>(
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }
}

enum MoveVariation {
    Create,
    CreateNodeFrom,
    Write,
    Invoke,
}

fn kernel_move_node_via_create_with_opened_substate(
    variation: MoveVariation,
) -> Result<(), RuntimeError> {
    let database = InMemorySubstateDatabase::standard();
    let mut track = Track::new(&database);
    let mut id_allocator = IdAllocator::new(Hash([0u8; Hash::LENGTH]));
    let mut callback = TestCallbackObject;
    let mut kernel = Kernel::new_no_refs(&mut track, &mut id_allocator, &mut callback);

    let child_id = {
        let child_id = kernel
            .kernel_allocate_node_id(EntityType::InternalKeyValueStore)
            .unwrap();
        let substates = btreemap!(
            PartitionNumber(0u8) => btreemap!(
                SubstateKey::Field(0u8) => IndexedScryptoValue::from_typed(&())
            )
        );
        kernel.kernel_create_node(child_id, substates).unwrap();
        kernel
            .kernel_open_substate(
                &child_id,
                PartitionNumber(0u8),
                &SubstateKey::Field(0u8),
                LockFlags::read_only(),
                (),
            )
            .unwrap();
        child_id
    };

    match variation {
        MoveVariation::Create => {
            let node_id = kernel
                .kernel_allocate_node_id(EntityType::GlobalAccount)
                .unwrap();
            let substates = btreemap!(
                PartitionNumber(0u8) => btreemap!(
                    SubstateKey::Field(0u8) => IndexedScryptoValue::from_typed(&Own(child_id))
                )
            );
            kernel.kernel_create_node(node_id, substates)?;
        }
        MoveVariation::CreateNodeFrom => {
            let node_id = kernel
                .kernel_allocate_node_id(EntityType::GlobalAccount)
                .unwrap();
            kernel.kernel_create_node_from(
                node_id,
                btreemap!(
                    PartitionNumber(0u8) => (child_id, PartitionNumber(0u8))
                ),
            )?;
        }
        MoveVariation::Write => {
            let node_id = kernel
                .kernel_allocate_node_id(EntityType::GlobalAccount)
                .unwrap();
            let substates = btreemap!(
                PartitionNumber(0u8) => btreemap!(
                    SubstateKey::Field(0u8) => IndexedScryptoValue::from_typed(&())
                )
            );
            kernel.kernel_create_node(node_id, substates)?;
            let handle = kernel.kernel_open_substate(
                &node_id,
                PartitionNumber(0u8),
                &SubstateKey::Field(0u8),
                LockFlags::MUTABLE,
                (),
            )?;
            kernel
                .kernel_write_substate(handle, IndexedScryptoValue::from_typed(&Own(child_id)))?;
        }
        MoveVariation::Invoke => {
            let invocation = KernelInvocation {
                call_frame_data: TestCallFrameData,
                args: IndexedScryptoValue::from_typed(&Own(child_id)),
            };
            kernel.kernel_invoke(Box::new(invocation))?;
        }
    }

    Ok(())
}

#[test]
fn test_kernel_move_node_via_create_with_opened_substate() {
    let result = kernel_move_node_via_create_with_opened_substate(MoveVariation::Create);
    assert_matches!(
        result,
        Err(RuntimeError::KernelError(KernelError::CallFrameError(
            CallFrameError::CreateNodeError(CreateNodeError::ProcessSubstateError(
                ProcessSubstateError::TakeNodeError(TakeNodeError::SubstateBorrowed(..))
            ))
        )))
    );
}

#[test]
fn test_kernel_move_node_via_write_with_opened_substate() {
    let result = kernel_move_node_via_create_with_opened_substate(MoveVariation::Write);
    assert_matches!(
        result,
        Err(RuntimeError::KernelError(KernelError::CallFrameError(
            CallFrameError::WriteSubstateError(WriteSubstateError::ProcessSubstateError(
                ProcessSubstateError::TakeNodeError(TakeNodeError::SubstateBorrowed(..))
            ))
        )))
    );
}

#[test]
fn test_kernel_move_node_via_move_partition_with_opened_substate() {
    let result = kernel_move_node_via_create_with_opened_substate(MoveVariation::CreateNodeFrom);
    println!("{:?}", result);
    assert_matches!(
        result,
        Err(RuntimeError::KernelError(KernelError::CallFrameError(
            CallFrameError::MovePartitionError(MovePartitionError::SubstateBorrowed(..))
        )))
    );
}

#[test]
fn test_kernel_move_node_via_invoke_with_opened_substate() {
    let result = kernel_move_node_via_create_with_opened_substate(MoveVariation::Invoke);
    assert_matches!(
        result,
        Err(RuntimeError::KernelError(KernelError::CallFrameError(
            CallFrameError::CreateFrameError(CreateFrameError::PassMessageError(
                PassMessageError::TakeNodeError(TakeNodeError::SubstateBorrowed(..))
            ))
        )))
    );
}

#[test]
fn kernel_close_substate_should_fail_if_opened_child_exists() {
    // Arrange
    let database = InMemorySubstateDatabase::standard();
    let mut track = Track::new(&database);
    let mut id_allocator = IdAllocator::new(Hash([0u8; Hash::LENGTH]));
    let mut callback = TestCallbackObject;
    let mut kernel = Kernel::new_no_refs(&mut track, &mut id_allocator, &mut callback);
    let mut create_node = || {
        let id = kernel
            .kernel_allocate_node_id(EntityType::InternalKeyValueStore)
            .unwrap();
        let substates = btreemap!(
            PartitionNumber(0u8) => btreemap!(
                SubstateKey::Field(0u8) => IndexedScryptoValue::from_typed(&())
            )
        );
        kernel.kernel_create_node(id, substates).unwrap();
        id
    };
    let node1 = create_node();
    let node2 = create_node();
    let handle1 = kernel
        .kernel_open_substate(
            &node1,
            PartitionNumber(0u8),
            &SubstateKey::Field(0u8),
            LockFlags::MUTABLE,
            (),
        )
        .unwrap();
    kernel
        .kernel_write_substate(handle1, IndexedScryptoValue::from_typed(&Own(node2)))
        .unwrap();
    kernel
        .kernel_open_substate(
            &node2,
            PartitionNumber(0u8),
            &SubstateKey::Field(0u8),
            LockFlags::MUTABLE,
            (),
        )
        .unwrap();

    // Act
    let result = kernel.kernel_close_substate(handle1);
    let error = result.expect_err("Should be an error");
    assert_matches!(
        error,
        RuntimeError::KernelError(KernelError::CallFrameError(
            CallFrameError::CloseSubstateError(CloseSubstateError::SubstateBorrowed(..))
        ))
    );
}
