use super::call_frame::{CallFrameMessage, StableReferenceType};
use crate::errors::*;
use crate::internal_prelude::*;
use crate::kernel::kernel_api::KernelInvocation;
use crate::kernel::kernel_api::{KernelApi, KernelInternalApi};
use crate::kernel::substate_io::SubstateDevice;
use crate::track::interface::{IOAccess, NodeSubstates};
use crate::track::{BootStore, CommitableSubstateStore, StoreCommitInfo, Track};
use crate::transaction::ResourcesUsage;
use radix_engine_interface::api::field_api::LockFlags;
use radix_substate_store_interface::db_key_mapper::SpreadPrefixKeyMapper;
use radix_substate_store_interface::interface::SubstateDatabase;
use radix_transactions::model::Executable;
use radix_transactions::prelude::PreAllocatedAddress;

pub trait CallFrameReferences {
    fn root() -> Self;
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
pub enum RefCheckEvent<'a> {
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
        value: &'a IndexedScryptoValue<'a>,
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
        value: &'a IndexedScryptoValue<'a>,
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
        &'a IndexedScryptoValue<'a>,
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
    fn from_rejection(executable: &Executable, reason: RejectionReason) -> Self;

    fn set_resource_usage(&mut self, resources_usage: ResourcesUsage);
}

/// Upper layer callback object which a kernel interacts with during execution
pub trait KernelCallbackObject: Sized {
    /// Data to be stored with each substate lock
    type LockData: Default + Clone;
    /// Data to be stored with every call frame
    type CallFrameData: CallFrameReferences;
    /// Initialization object
    type Init: Clone;
    /// Output to be returned at the end of execution
    type ExecutionOutput;
    /// Final receipt to be created after transaction execution
    type Receipt: ExecutionReceipt;

    /// Create the callback object (system layer) with data loaded from the substate store
    fn init<S: BootStore + CommitableSubstateStore>(
        store: &mut S,
        executable: &Executable,
        init: Self::Init,
    ) -> Result<Self, RejectionReason>;

    /// Verifies and returns the type of a given reference used during boot
    fn verify_boot_ref_value(
        &mut self,
        node_id: &NodeId,
        value: &IndexedScryptoValue,
    ) -> Result<StableReferenceType, BootloadingError>;

    /// Start execution
    fn start<Y: KernelApi<Self>>(
        api: &mut Y,
        manifest_encoded_instructions: &[u8],
        pre_allocated_addresses: &Vec<PreAllocatedAddress>,
        references: &IndexSet<Reference>,
        blobs: &IndexMap<Hash, Vec<u8>>,
    ) -> Result<Self::ExecutionOutput, RuntimeError>;

    /// Finish execution
    fn finish(&mut self, store_commit_info: StoreCommitInfo) -> Result<(), RuntimeError>;

    /// Create final receipt
    fn create_receipt<S: SubstateDatabase>(
        self,
        track: Track<S, SpreadPrefixKeyMapper>,
        executable: &Executable,
        result: Result<Self::ExecutionOutput, TransactionExecutionError>,
    ) -> Self::Receipt;

    /// Callback before a node is pinned to it's device
    fn on_pin_node(&mut self, node_id: &NodeId) -> Result<(), RuntimeError>;

    /// Callbacks before/on-io-access/after a new node is created
    fn on_create_node<Y: KernelInternalApi<Self>>(
        api: &mut Y,
        event: CreateNodeEvent,
    ) -> Result<(), RuntimeError>;

    /// Callbacks before/on-io-access/after a node is dropped
    fn on_drop_node<Y: KernelInternalApi<Self>>(
        api: &mut Y,
        event: DropNodeEvent,
    ) -> Result<(), RuntimeError>;

    /// Callback when a module is moved
    fn on_move_module<Y: KernelInternalApi<Self>>(
        api: &mut Y,
        event: MoveModuleEvent,
    ) -> Result<(), RuntimeError>;

    /// Callback before/on-io-access/after a substate is opened
    fn on_open_substate<Y: KernelInternalApi<Self>>(
        api: &mut Y,
        event: OpenSubstateEvent,
    ) -> Result<(), RuntimeError>;

    /// Callback before a substate is closed
    fn on_close_substate<Y: KernelInternalApi<Self>>(
        api: &mut Y,
        event: CloseSubstateEvent,
    ) -> Result<(), RuntimeError>;

    /// Callback before a substate is read
    fn on_read_substate<Y: KernelInternalApi<Self>>(
        api: &mut Y,
        event: ReadSubstateEvent,
    ) -> Result<(), RuntimeError>;

    /// Callback before a substate is written to
    fn on_write_substate<Y: KernelInternalApi<Self>>(
        api: &mut Y,
        event: WriteSubstateEvent,
    ) -> Result<(), RuntimeError>;

    /// Callback before/on-io-access a substate is set
    fn on_set_substate(&mut self, event: SetSubstateEvent) -> Result<(), RuntimeError>;

    /// Callback before/on-io-access a substate is removed
    fn on_remove_substate(&mut self, event: RemoveSubstateEvent) -> Result<(), RuntimeError>;

    /// Callback before/on-io-access a key scan occurs
    fn on_scan_keys(&mut self, event: ScanKeysEvent) -> Result<(), RuntimeError>;

    /// Callback before/on-io-access a substate drain occurs
    fn on_drain_substates(&mut self, event: DrainSubstatesEvent) -> Result<(), RuntimeError>;

    /// Callback before/on-io-access a sorted substate scan occurs
    fn on_scan_sorted_substates(
        &mut self,
        event: ScanSortedSubstatesEvent,
    ) -> Result<(), RuntimeError>;

    /// Callback before an invocation and a new call frame is created
    fn before_invoke<Y: KernelApi<Self>>(
        invocation: &KernelInvocation<Self::CallFrameData>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    /// Callback after a new call frame is created for a new invocation
    fn on_execution_start<Y: KernelApi<Self>>(api: &mut Y) -> Result<(), RuntimeError>;

    /// Callback on invocation. This is where the callback object should execute application logic.
    fn invoke_upstream<Y: KernelApi<Self>>(
        args: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedOwnedScryptoValue, RuntimeError>;

    /// Callback after invocation during call frame cleanup and nodes are still owned by the executed
    /// call frame
    fn auto_drop<Y: KernelApi<Self>>(nodes: Vec<NodeId>, api: &mut Y) -> Result<(), RuntimeError>;

    /// Callback right after execution of invocation and where call of execution still exists
    fn on_execution_finish<Y: KernelApi<Self>>(
        message: &CallFrameMessage,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    /// Callback after an invocation and where invocation's call frame has already been destroyed
    fn after_invoke<Y: KernelApi<Self>>(
        output: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    /// Callback before node id allocation
    fn on_allocate_node_id<Y: KernelApi<Self>>(
        entity_type: EntityType,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    /// Callback before a substate is marked as transient
    fn on_mark_substate_as_transient(
        &mut self,
        node_id: &NodeId,
        partition_number: &PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Result<(), RuntimeError>;

    /// Callback when a substate does not exist
    fn on_substate_lock_fault<Y: KernelApi<Self>>(
        node_id: NodeId,
        partition_num: PartitionNumber,
        offset: &SubstateKey,
        api: &mut Y,
    ) -> Result<bool, RuntimeError>;

    /// Callback before a node is dropped
    fn on_drop_node_mut<Y: KernelApi<Self>>(
        node_id: &NodeId,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;
}
