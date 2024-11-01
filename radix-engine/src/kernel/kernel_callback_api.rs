use super::call_frame::{CallFrameInit, CallFrameMessage};
use crate::errors::*;
use crate::internal_prelude::*;
use crate::kernel::kernel_api::KernelInvocation;
use crate::kernel::kernel_api::{KernelApi, KernelInternalApi};
use crate::kernel::substate_io::SubstateDevice;
use crate::track::interface::{IOAccess, NodeSubstates};
use crate::track::*;
use crate::transaction::ResourcesUsage;
use radix_engine_interface::api::field_api::LockFlags;
use radix_substate_store_interface::interface::SubstateDatabase;
use radix_transactions::model::ExecutableTransaction;

pub trait CallFrameReferences {
    fn global_references(&self) -> Vec<NodeId>;
    fn direct_access_references(&self) -> Vec<NodeId>;
    fn stable_transient_references(&self) -> Vec<NodeId>;

    fn len(&self) -> usize;
}

// TODO: Replace Events with separate callback functions
#[derive(Debug)]
pub enum CreateNodeEvent<'a> {
    Start(&'a NodeId, &'a NodeSubstates),
    IOAccess(&'a IOAccess),
    End(&'a NodeId),
}

#[derive(Debug)]
pub enum DropNodeEvent<'a> {
    Start(&'a NodeId),
    IOAccess(&'a IOAccess),
    End(&'a NodeId, &'a NodeSubstates),
}

#[derive(Debug)]
pub enum CheckReferenceEvent<'a> {
    IOAccess(&'a IOAccess),
}

#[derive(Debug)]
pub enum MoveModuleEvent<'a> {
    IOAccess(&'a IOAccess),
}

#[derive(Debug)]
pub enum OpenSubstateEvent<'a> {
    Start {
        node_id: &'a NodeId,
        partition_num: &'a PartitionNumber,
        substate_key: &'a SubstateKey,
        flags: &'a LockFlags,
    },
    IOAccess(&'a IOAccess),
    End {
        handle: SubstateHandle,
        node_id: &'a NodeId,
        size: usize,
    },
}

#[derive(Debug)]
pub enum ReadSubstateEvent<'a> {
    OnRead {
        handle: SubstateHandle,
        value: &'a IndexedScryptoValue,
        device: SubstateDevice,
    },
    IOAccess(&'a IOAccess),
}

impl<'a> ReadSubstateEvent<'a> {
    pub fn is_about_heap(&self) -> bool {
        match self {
            ReadSubstateEvent::OnRead { device, .. } => matches!(device, SubstateDevice::Heap),
            ReadSubstateEvent::IOAccess(access) => match access {
                IOAccess::ReadFromDb(_, _) => false,
                IOAccess::ReadFromDbNotFound(_) => false,
                IOAccess::TrackSubstateUpdated { .. } => false,
                IOAccess::HeapSubstateUpdated { .. } => true,
            },
        }
    }
}

#[derive(Debug)]
pub enum WriteSubstateEvent<'a> {
    Start {
        handle: SubstateHandle,
        value: &'a IndexedScryptoValue,
    },
    IOAccess(&'a IOAccess),
}

#[derive(Debug)]
pub enum CloseSubstateEvent {
    Start(SubstateHandle),
}

#[derive(Debug)]
pub enum SetSubstateEvent<'a> {
    Start(
        &'a NodeId,
        &'a PartitionNumber,
        &'a SubstateKey,
        &'a IndexedScryptoValue,
    ),
    IOAccess(&'a IOAccess),
}

#[derive(Debug)]
pub enum RemoveSubstateEvent<'a> {
    Start(&'a NodeId, &'a PartitionNumber, &'a SubstateKey),
    IOAccess(&'a IOAccess),
}

#[derive(Debug)]
pub enum ScanKeysEvent<'a> {
    Start,
    IOAccess(&'a IOAccess),
}

#[derive(Debug)]
pub enum DrainSubstatesEvent<'a> {
    Start(u32),
    IOAccess(&'a IOAccess),
}

#[derive(Debug)]
pub enum ScanSortedSubstatesEvent<'a> {
    Start,
    IOAccess(&'a IOAccess),
}

/// A receipt created from executing a transaction
pub trait ExecutionReceipt {
    fn set_resource_usage(&mut self, resources_usage: ResourcesUsage);
}

/// A transaction which has a unique id, useful for creating an IdAllocator which
/// requires a unique input
pub trait UniqueSeed {
    fn unique_seed_for_id_allocator(&self) -> Hash;
}

impl UniqueSeed for ExecutableTransaction {
    fn unique_seed_for_id_allocator(&self) -> Hash {
        *self.unique_hash()
    }
}

pub trait KernelTransactionExecutor: KernelCallbackObject {
    /// Initialization object
    type Init;
    /// The transaction object
    type Executable: UniqueSeed;
    /// Output to be returned at the end of execution
    type ExecutionOutput;
    /// Final receipt to be created after transaction execution
    type Receipt: ExecutionReceipt;

    /// Create the callback object (system layer) and the initial call frame configuration for each intent
    fn init(
        store: &mut impl CommitableSubstateStore,
        executable: &Self::Executable,
        init: Self::Init,
        always_visible_global_nodes: &'static IndexSet<NodeId>,
    ) -> Result<(Self, Vec<CallFrameInit<Self::CallFrameData>>), Self::Receipt>;

    /// Start execution
    fn execute<Y: KernelApi<CallbackObject = Self>>(
        api: &mut Y,
        executable: &Self::Executable,
    ) -> Result<Self::ExecutionOutput, RuntimeError>;

    /// Finish execution
    fn finalize(
        &mut self,
        executable: &Self::Executable,
        store_commit_info: StoreCommitInfo,
    ) -> Result<(), RuntimeError>;

    /// Create final receipt
    fn create_receipt<S: SubstateDatabase>(
        self,
        track: Track<S>,
        result: Result<Self::ExecutionOutput, TransactionExecutionError>,
    ) -> Self::Receipt;
}

/// Upper layer callback object which a kernel interacts with during execution
pub trait KernelCallbackObject: Sized {
    /// Data to be stored with each substate lock
    type LockData: Default + Clone;
    /// Data to be stored with every call frame
    type CallFrameData: CallFrameReferences;

    /// Callback before a node is pinned to the heap
    fn on_pin_node<Y: KernelInternalApi<System = Self>>(
        node_id: &NodeId,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    /// Callbacks before/on-io-access/after a new node is created
    fn on_create_node<Y: KernelInternalApi<System = Self>>(
        event: CreateNodeEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    /// Callbacks before/on-io-access/after a node is dropped
    fn on_drop_node<Y: KernelInternalApi<System = Self>>(
        event: DropNodeEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    /// Callback when a module is moved
    fn on_move_module<Y: KernelInternalApi<System = Self>>(
        event: MoveModuleEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    /// Callback before/on-io-access/after a substate is opened
    fn on_open_substate<Y: KernelInternalApi<System = Self>>(
        event: OpenSubstateEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    /// Callback before a substate is closed
    fn on_close_substate<Y: KernelInternalApi<System = Self>>(
        event: CloseSubstateEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    /// Callback before a substate is read
    fn on_read_substate<Y: KernelInternalApi<System = Self>>(
        event: ReadSubstateEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    /// Callback before a substate is written to
    fn on_write_substate<Y: KernelInternalApi<System = Self>>(
        event: WriteSubstateEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    /// Callback before/on-io-access a substate is set
    fn on_set_substate<Y: KernelInternalApi<System = Self>>(
        event: SetSubstateEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    /// Callback before/on-io-access a substate is removed
    fn on_remove_substate<Y: KernelInternalApi<System = Self>>(
        event: RemoveSubstateEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    /// Callback before/on-io-access a key scan occurs
    fn on_scan_keys<Y: KernelInternalApi<System = Self>>(
        event: ScanKeysEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    /// Callback before/on-io-access a substate drain occurs
    fn on_drain_substates<Y: KernelInternalApi<System = Self>>(
        event: DrainSubstatesEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    /// Callback before/on-io-access a sorted substate scan occurs
    fn on_scan_sorted_substates<Y: KernelInternalApi<System = Self>>(
        event: ScanSortedSubstatesEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    /// Callback before an invocation and a new call frame is created
    fn before_invoke<Y: KernelApi<CallbackObject = Self>>(
        invocation: &KernelInvocation<Self::CallFrameData>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    /// Callback after a new call frame is created for a new invocation
    fn on_execution_start<Y: KernelInternalApi<System = Self>>(
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    /// Callback on invocation. This is where the callback object should execute application logic.
    fn invoke_upstream<Y: KernelApi<CallbackObject = Self>>(
        args: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>;

    /// Callback after invocation during call frame cleanup and nodes are still owned by the executed
    /// call frame
    fn auto_drop<Y: KernelApi<CallbackObject = Self>>(
        nodes: Vec<NodeId>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    /// Callback right after execution of invocation and where call of execution still exists
    fn on_execution_finish<Y: KernelInternalApi<System = Self>>(
        message: &CallFrameMessage,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    /// Callback after an invocation and where invocation's call frame has already been destroyed
    fn after_invoke<Y: KernelApi<CallbackObject = Self>>(
        output: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    /// Callback before node id allocation
    fn on_allocate_node_id<Y: KernelInternalApi<System = Self>>(
        entity_type: EntityType,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    /// Callback before a substate is marked as transient
    fn on_mark_substate_as_transient<Y: KernelInternalApi<System = Self>>(
        node_id: &NodeId,
        partition_number: &PartitionNumber,
        substate_key: &SubstateKey,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    /// Callback when a substate does not exist
    fn on_substate_lock_fault<Y: KernelApi<CallbackObject = Self>>(
        node_id: NodeId,
        partition_num: PartitionNumber,
        offset: &SubstateKey,
        api: &mut Y,
    ) -> Result<bool, RuntimeError>;

    /// Callback before a node is dropped
    fn on_drop_node_mut<Y: KernelApi<CallbackObject = Self>>(
        node_id: &NodeId,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    fn on_get_stack_id<Y: KernelInternalApi<System = Self>>(
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    fn on_switch_stack<Y: KernelInternalApi<System = Self>>(
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    fn on_send_to_stack<Y: KernelInternalApi<System = Self>>(
        value: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    fn on_set_call_frame_data<Y: KernelInternalApi<System = Self>>(
        data: &Self::CallFrameData,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    fn on_get_owned_nodes<Y: KernelInternalApi<System = Self>>(
        api: &mut Y,
    ) -> Result<(), RuntimeError>;
}
