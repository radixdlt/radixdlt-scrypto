use std::cmp::max;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;
use radix_engine::errors::{CallFrameError, KernelError, RuntimeError};
use radix_engine::kernel::call_frame::{
    CallFrameMessage, CreateFrameError, CreateNodeError, PassMessageError, ProcessSubstateError,
    TakeNodeError,
};
use radix_engine::kernel::id_allocator::IdAllocator;
use radix_engine::kernel::kernel::{Kernel, KernelBoot};
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
use radix_engine::track::{CommitableSubstateStore, Track};
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

struct KernelFuzzer {
    rng: ChaCha8Rng,
    allocated_nodes: Vec<NodeId>,
    nodes: Vec<NodeId>,
    handles: Vec<SubstateHandle>,
}

impl KernelFuzzer {
    fn new(seed: u64) -> Self {
        KernelFuzzer {
            rng: ChaCha8Rng::seed_from_u64(seed),
            allocated_nodes: Vec::new(),
            nodes: Vec::new(),
            handles: Vec::new(),
        }
    }

    fn add_allocated_node(&mut self, node_id: NodeId) {
        self.allocated_nodes.push(node_id);
    }

    fn add_node(&mut self, node_id: NodeId) {
        self.nodes.push(node_id);
    }

    fn next_allocated_node(&mut self) -> Option<NodeId> {
        if self.allocated_nodes.is_empty() {
            None
        } else {
            let index = self.rng.gen_range(0usize..self.allocated_nodes.len());
            let node_id = self.allocated_nodes.remove(index);
            self.nodes.push(node_id);
            Some(node_id)
        }
    }

    fn next_node(&mut self) -> Option<NodeId> {
        if self.rng.gen_bool(0.5) {
            if self.nodes.is_empty() {
                None
            } else {
                let index = self.rng.gen_range(0usize..self.nodes.len());
                Some(self.nodes[index])
            }
        } else {
            self.next_allocated_node()
        }
    }

    fn add_handle(&mut self, handle: SubstateHandle) {
        self.handles.push(handle);
    }

    fn next_handle(&mut self) -> Option<SubstateHandle> {
        if self.handles.is_empty() {
            None
        } else {
            let index = self.rng.gen_range(0usize..self.handles.len());
            Some(self.handles[index])
        }
    }

    fn next_value(&mut self) -> IndexedScryptoValue {
        if let Some(child_id) = self.next_node() {
            IndexedScryptoValue::from_typed(&Own(child_id))
        } else {
            IndexedScryptoValue::from_typed(&())
        }
    }

    fn next_entity_type(&mut self) -> EntityType {
        if self.rng.gen_bool(0.5) {
            EntityType::InternalKeyValueStore
        } else {
            EntityType::GlobalAccount
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
enum KernelFuzzAction {
    Allocate,
    CreateNode,
    PinNode,
    DropNode,
    Invoke,
    MovePartition,
    OpenSubstate,
    ReadSubstate,
    WriteSubstate,
    CloseSubstate
}

impl KernelFuzzAction {
    fn execute<S>(&self, fuzzer: &mut KernelFuzzer, kernel: &mut Kernel<'_, TestCallbackObject, S>)
    -> Result<bool, RuntimeError>
    where
        S: CommitableSubstateStore,
    {
        match self {
            KernelFuzzAction::Allocate => {
                let node_id = kernel
                    .kernel_allocate_node_id(fuzzer.next_entity_type())?;
                fuzzer.add_allocated_node(node_id);
                return Ok(false);
            }
            KernelFuzzAction::CreateNode => {
                if let Some(node_id) = fuzzer.next_allocated_node() {
                    let value = fuzzer.next_value();
                    let substates = btreemap!(
                        PartitionNumber(0u8) => btreemap!(
                            SubstateKey::Field(0u8) => value
                        )
                    );
                    kernel.kernel_create_node(node_id, substates)?;
                    return Ok(false);
                }
                return Ok(true);
            }
            KernelFuzzAction::PinNode => {
                if let Some(node_id) = fuzzer.next_node() {
                    kernel.kernel_pin_node(node_id)?;
                    return Ok(false);
                }

                return Ok(true);
            }
            KernelFuzzAction::DropNode => {
                if let Some(node_id) = fuzzer.next_node() {
                    kernel.kernel_drop_node(&node_id)?;
                    return Ok(false);
                }

                return Ok(true);
            }
            KernelFuzzAction::MovePartition => {
                if let Some(src) = fuzzer.next_node().filter(|n| !n.is_global()) {
                    if let Some(dest) = fuzzer.next_node() {
                        kernel.kernel_move_partition(
                            &src,
                            PartitionNumber(0u8),
                            &dest,
                            PartitionNumber(0u8),
                        )?;

                        return Ok(false);
                    }
                }

                return Ok(true);
            }
            KernelFuzzAction::Invoke => {
                if let Some(node_id) = fuzzer.next_node() {
                    let invocation = KernelInvocation {
                        call_frame_data: TestCallFrameData,
                        args: IndexedScryptoValue::from_typed(&Own(node_id)),
                    };
                    kernel.kernel_invoke(Box::new(invocation))?;
                    return Ok(false);
                }

                return Ok(true);
            }
            KernelFuzzAction::OpenSubstate => {
                if let Some(node_id) = fuzzer.next_node() {
                    let handle = kernel
                        .kernel_open_substate(
                            &node_id,
                            PartitionNumber(0u8),
                            &SubstateKey::Field(0u8),
                            LockFlags::read_only(),
                            (),
                        )?;
                    fuzzer.add_handle(handle);
                    return Ok(false);
                }

                return Ok(true);
            }
            KernelFuzzAction::ReadSubstate => {
                if let Some(handle) = fuzzer.next_handle() {
                    kernel.kernel_read_substate(handle)?;
                    return Ok(false);
                }

                return Ok(true);
            }
            KernelFuzzAction::WriteSubstate => {
                if let Some(handle) = fuzzer.next_handle() {
                    let value = fuzzer.next_value();
                    kernel.kernel_write_substate(handle, value)?;
                    return Ok(false);
                }

                return Ok(true);
            }
            KernelFuzzAction::CloseSubstate => {
                if let Some(handle) = fuzzer.next_handle() {
                    kernel.kernel_close_substate(handle)?;
                    return Ok(false);
                }

                return Ok(true);
            }
        }
    }
}

fn kernel_fuzz(seed: u64) -> u32 {
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

    let mut fuzzer = KernelFuzzer::new(seed);
    let mut success_count = 0u32;

    loop {
        let action = KernelFuzzAction::from_repr(fuzzer.rng.gen_range(0u8..=9u8)).unwrap();
        match action.execute(&mut fuzzer, &mut kernel) {
            Ok(trivial) => {
                if !trivial {
                    success_count += 1;
                }
            }
            Err(..) => {
                return success_count;
            }
        }
    }
}

#[test]
fn test_kernel_fuzz() {
    let mut highest_success_count = 0u32;
    for seed in 0u64..1000000u64 {
        let success_count = kernel_fuzz(seed);
        highest_success_count = max(success_count, highest_success_count);
    }

    println!("Highest success count: {:?}", highest_success_count);
}
