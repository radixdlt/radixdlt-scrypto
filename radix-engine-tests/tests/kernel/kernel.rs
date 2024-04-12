use radix_common::prelude::*;
use radix_engine::errors::{BootloadingError, CallFrameError, KernelError, RejectionReason, RuntimeError};
use radix_engine::kernel::call_frame::{
    CallFrameMessage, CloseSubstateError, CreateFrameError, CreateNodeError, MovePartitionError,
    PassMessageError, ProcessSubstateError, TakeNodeError, WriteSubstateError,
};
use radix_engine::kernel::id_allocator::IdAllocator;
use radix_engine::kernel::kernel::BootLoader;
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
use radix_engine::track::{BootStore, StoreCommitInfo, Track};
use radix_engine_interface::prelude::*;
use radix_substate_store_impls::memory_db::InMemorySubstateDatabase;
use radix_substate_store_interface::db_key_mapper::SpreadPrefixKeyMapper;
use radix_substate_store_interface::interface::SubstateDatabase;
use radix_transactions::model::{Executable, PreAllocatedAddress};

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
    type InitInput = ();

    fn init<S: BootStore>(_store: &S, _executable: &Executable, _init_input: Self::InitInput) -> Result<Self, BootloadingError> {
        Ok(Self)
    }

    fn init2<S: SubstateDatabase>(&self, _track: &mut Track<S, SpreadPrefixKeyMapper>, _executable: &Executable) -> Result<(), RejectionReason> {
        Ok(())
    }

    fn start<Y>(
        _api: &mut Y,
        _manifest_encoded_instructions: &[u8],
        _pre_allocated_addresses: &Vec<PreAllocatedAddress>,
        _references: &IndexSet<Reference>,
        _blobs: &IndexMap<Hash, Vec<u8>>,
    ) -> Result<Vec<u8>, RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        unreachable!()
    }

    fn on_teardown<Y>(_api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        Ok(())
    }

    fn on_teardown2(&mut self, _store_commit_info: StoreCommitInfo) -> Result<(), RuntimeError> {
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

enum MoveVariation {
    Create,
    CreateNodeFrom,
    Write,
    Invoke,
}

fn kernel_move_node_via_create_with_opened_substate(
    variation: MoveVariation,
) -> Result<(), RuntimeError> {
    let mut id_allocator = IdAllocator::new(Hash([0u8; Hash::LENGTH]));
    let database = InMemorySubstateDatabase::standard();
    let mut track = Track::<InMemorySubstateDatabase, SpreadPrefixKeyMapper>::new(&database);
    let mut callback = TestCallbackObject;
    let mut boot_loader = BootLoader {
        id_allocator: &mut id_allocator,
        callback: &mut callback,
        store: &mut track,
    };
    let mut kernel = boot_loader.boot().unwrap();

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
fn test_kernel_move_node_via_write_with_opened_substate() {
    let result = kernel_move_node_via_create_with_opened_substate(MoveVariation::Write);
    assert!(matches!(
        result,
        Err(RuntimeError::KernelError(KernelError::CallFrameError(
            CallFrameError::WriteSubstateError(WriteSubstateError::ProcessSubstateError(
                ProcessSubstateError::TakeNodeError(TakeNodeError::SubstateBorrowed(..))
            ))
        )))
    ));
}

#[test]
fn test_kernel_move_node_via_move_partition_with_opened_substate() {
    let result = kernel_move_node_via_create_with_opened_substate(MoveVariation::CreateNodeFrom);
    println!("{:?}", result);
    assert!(matches!(
        result,
        Err(RuntimeError::KernelError(KernelError::CallFrameError(
            CallFrameError::MovePartitionError(MovePartitionError::SubstateBorrowed(..))
        )))
    ));
}

#[test]
fn test_kernel_move_node_via_invoke_with_opened_substate() {
    let result = kernel_move_node_via_create_with_opened_substate(MoveVariation::Invoke);
    assert!(matches!(
        result,
        Err(RuntimeError::KernelError(KernelError::CallFrameError(
            CallFrameError::CreateFrameError(CreateFrameError::PassMessageError(
                PassMessageError::TakeNodeError(TakeNodeError::SubstateBorrowed(..))
            ))
        )))
    ));
}

#[test]
fn kernel_close_substate_should_fail_if_opened_child_exists() {
    // Arrange
    let mut id_allocator = IdAllocator::new(Hash([0u8; Hash::LENGTH]));
    let database = InMemorySubstateDatabase::standard();
    let mut track = Track::<InMemorySubstateDatabase, SpreadPrefixKeyMapper>::new(&database);
    let mut callback = TestCallbackObject;
    let mut boot_loader = BootLoader {
        id_allocator: &mut id_allocator,
        callback: &mut callback,
        store: &mut track,
    };
    let mut kernel = boot_loader.boot().unwrap();
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
    assert!(matches!(
        error,
        RuntimeError::KernelError(KernelError::CallFrameError(
            CallFrameError::CloseSubstateError(CloseSubstateError::SubstateBorrowed(..))
        ))
    ));
}
