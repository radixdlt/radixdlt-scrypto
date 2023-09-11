use radix_engine::errors::{CallFrameError, KernelError, RuntimeError};
use radix_engine::kernel::call_frame::{
    CallFrameMessage, CreateFrameError, CreateNodeError, PassMessageError, ProcessSubstateError,
    TakeNodeError,
};
use radix_engine::kernel::id_allocator::IdAllocator;
use radix_engine::kernel::kernel::KernelBoot;
use radix_engine::kernel::kernel_api::{
    KernelApi, KernelInternalApi, KernelInvocation, KernelInvokeApi, KernelNodeApi,
    KernelSubstateApi,
};
use radix_engine::kernel::kernel_callback_api::{
    CallFrameReferences, CloseSubstateEvent, CreateNodeEvent, DrainSubstatesEvent, DropNodeEvent,
    KernelCallbackObject, MoveModuleEvent, OpenSubstateEvent, ReadSubstateEvent,
    RemoveSubstateEvent, ScanKeysEvent, ScanSortedSubstatesEvent, SetSubstateEvent,
    WriteSubstateEvent,
};
use radix_engine::track::Track;
use radix_engine::types::*;
use radix_engine_store_interface::db_key_mapper::SpreadPrefixKeyMapper;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;

struct TestCallFrameData;

impl CallFrameReferences for TestCallFrameData {
    fn root() -> Self {
        TestCallFrameData
    }

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

    fn on_init<Y>(_api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        Ok(())
    }

    fn on_teardown<Y>(_api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        Ok(())
    }

    fn on_pin_node(&mut self, _node_id: &NodeId) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_create_node<Y>(_api: &mut Y, _event: CreateNodeEvent) -> Result<(), RuntimeError>
    where
        Y: KernelInternalApi<Self>,
    {
        Ok(())
    }

    fn on_drop_node<Y>(_api: &mut Y, _event: DropNodeEvent) -> Result<(), RuntimeError>
    where
        Y: KernelInternalApi<Self>,
    {
        Ok(())
    }

    fn on_move_module<Y>(_api: &mut Y, _event: MoveModuleEvent) -> Result<(), RuntimeError>
    where
        Y: KernelInternalApi<Self>,
    {
        Ok(())
    }

    fn on_open_substate<Y>(_api: &mut Y, _event: OpenSubstateEvent) -> Result<(), RuntimeError>
    where
        Y: KernelInternalApi<Self>,
    {
        Ok(())
    }

    fn on_close_substate<Y>(_api: &mut Y, _event: CloseSubstateEvent) -> Result<(), RuntimeError>
    where
        Y: KernelInternalApi<Self>,
    {
        Ok(())
    }

    fn on_read_substate<Y>(_api: &mut Y, _event: ReadSubstateEvent) -> Result<(), RuntimeError>
    where
        Y: KernelInternalApi<Self>,
    {
        Ok(())
    }

    fn on_write_substate<Y>(_api: &mut Y, _event: WriteSubstateEvent) -> Result<(), RuntimeError>
    where
        Y: KernelInternalApi<Self>,
    {
        Ok(())
    }

    fn on_set_substate(&mut self, _event: SetSubstateEvent) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_remove_substate(&mut self, _event: RemoveSubstateEvent) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_scan_keys(&mut self, _event: ScanKeysEvent) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_drain_substates(&mut self, _event: DrainSubstatesEvent) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_scan_sorted_substates(
        &mut self,
        _event: ScanSortedSubstatesEvent,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn before_invoke<Y>(
        _invocation: &KernelInvocation<Self::CallFrameData>,
        _api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        Ok(())
    }

    fn after_invoke<Y>(_output: &IndexedScryptoValue, _api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        Ok(())
    }

    fn on_execution_start<Y>(_api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        Ok(())
    }

    fn on_execution_finish<Y>(_message: &CallFrameMessage, _api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        Ok(())
    }

    fn on_allocate_node_id<Y>(_entity_type: EntityType, _api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        Ok(())
    }

    fn invoke_upstream<Y>(
        args: &IndexedScryptoValue,
        _api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        Ok(args.clone())
    }

    fn auto_drop<Y>(_nodes: Vec<NodeId>, _api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        Ok(())
    }

    fn on_mark_substate_as_transient(
        &mut self,
        _node_id: &NodeId,
        _partition_number: &PartitionNumber,
        _substate_key: &SubstateKey,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_substate_lock_fault<Y>(
        _node_id: NodeId,
        _partition_num: PartitionNumber,
        _offset: &SubstateKey,
        _api: &mut Y,
    ) -> Result<bool, RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        Ok(false)
    }

    fn on_drop_node_mut<Y>(_node_id: &NodeId, _api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        Ok(())
    }

    fn on_move_node<Y>(
        _node_id: &NodeId,
        _is_moving_down: bool,
        _is_to_barrier: bool,
        _destination_blueprint_id: Option<BlueprintId>,
        _api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        Ok(())
    }
}

enum MoveNodeVariation {
    Create,
    Invoke,
}

fn kernel_move_node_via_create_with_opened_substate(
    variation: MoveNodeVariation,
) -> Result<(), RuntimeError> {
    let mut id_allocator = IdAllocator::new(Hash([0u8; Hash::LENGTH]));
    let database = InMemorySubstateDatabase::standard();
    let mut track = Track::<InMemorySubstateDatabase, SpreadPrefixKeyMapper>::new(&database);
    let mut callback = TestCallbackObject;
    let mut kernel_boot = KernelBoot {
        id_allocator: &mut id_allocator,
        callback: &mut callback,
        store: &mut track,
    };
    let mut kernel = kernel_boot.create_kernel_for_test_only();

    let (child_id, handle) = {
        let child_id = kernel
            .kernel_allocate_node_id(EntityType::InternalKeyValueStore)
            .unwrap();
        let substates = btreemap!(
            PartitionNumber(0u8) => btreemap!(
                SubstateKey::Field(0u8) => IndexedScryptoValue::from_typed(&())
            )
        );
        kernel.kernel_create_node(child_id, substates).unwrap();
        let handle = kernel
            .kernel_open_substate(
                &child_id,
                PartitionNumber(0u8),
                &SubstateKey::Field(0u8),
                LockFlags::read_only(),
                (),
            )
            .unwrap();
        (child_id, handle)
    };

    match variation {
        MoveNodeVariation::Create => {
            let node_id = kernel
                .kernel_allocate_node_id(EntityType::GlobalAccount)
                .unwrap();
            let substates = btreemap!(
                PartitionNumber(0u8) => btreemap!(
                    SubstateKey::Field(0u8) => IndexedScryptoValue::from_typed(&Own(child_id))
                )
            );
            kernel.kernel_create_node(node_id, substates)?;
            kernel.kernel_read_substate(handle)?;
        }
        MoveNodeVariation::Invoke => {
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
    let result = kernel_move_node_via_create_with_opened_substate(MoveNodeVariation::Create);
    assert!(matches!(
        result,
        Err(RuntimeError::KernelError(KernelError::CallFrameError(
            CallFrameError::CreateNodeError(CreateNodeError::ProcessSubstateError(
                ProcessSubstateError::TakeNodeError(TakeNodeError::SubstateBorrowed(..))
            ))
        )))
    ));
}

#[test]
fn test_kernel_move_node_via_invoke_with_opened_substate() {
    let result = kernel_move_node_via_create_with_opened_substate(MoveNodeVariation::Invoke);
    assert!(matches!(
        result,
        Err(RuntimeError::KernelError(KernelError::CallFrameError(
            CallFrameError::CreateFrameError(CreateFrameError::PassMessageError(
                PassMessageError::TakeNodeError(TakeNodeError::SubstateBorrowed(..))
            ))
        )))
    ));
}
