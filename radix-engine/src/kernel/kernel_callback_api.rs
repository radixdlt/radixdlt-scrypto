use super::call_frame::CallFrameMessage;
use crate::errors::*;
use crate::internal_prelude::*;
use crate::kernel::kernel_api::KernelInvocation;
use crate::kernel::kernel_api::{KernelApi, KernelInternalApi};
use crate::kernel::substate_io::SubstateDevice;
use crate::track::interface::{IOAccess, NodeSubstates};
use crate::track::{BootStore, StoreCommitInfo, Track};
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use radix_substate_store_interface::db_key_mapper::SpreadPrefixKeyMapper;
use radix_substate_store_interface::interface::SubstateDatabase;
use radix_transactions::model::Executable;
use radix_transactions::prelude::PreAllocatedAddress;
use crate::transaction::{CostingParameters, ResourcesUsage, TransactionFeeDetails, TransactionFeeSummary, TransactionReceipt, TransactionResult};

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

pub trait KernelCallbackObject: Sized {
    type LockData: Default + Clone;
    type CallFrameData: CallFrameReferences;
    type InitInput: Clone;

    /// Initialize the system layer with data loaded from the substate store
    fn init<S: BootStore>(
        store: &S,
        executable: &Executable,
        init_input: Self::InitInput,
    ) -> Result<Self, BootloadingError>;

    fn init2<S: SubstateDatabase>(
        &self,
        track: &mut Track<S, SpreadPrefixKeyMapper>,
        executable: &Executable,
    ) -> Result<(), RejectionReason>;

    fn start<Y>(
        api: &mut Y,
        manifest_encoded_instructions: &[u8],
        pre_allocated_addresses: &Vec<PreAllocatedAddress>,
        references: &IndexSet<Reference>,
        blobs: &IndexMap<Hash, Vec<u8>>,
    ) -> Result<Vec<u8>, RuntimeError>
        where
            Y: KernelApi<Self>;

    fn on_teardown<Y>(api: &mut Y) -> Result<(), RuntimeError>
        where
            Y: KernelApi<Self>;

    fn on_teardown2(&mut self, store_commit_info: StoreCommitInfo) -> Result<(), RuntimeError>;

    fn on_teardown3<S: SubstateDatabase>(
        self,
        track: Track<S, SpreadPrefixKeyMapper>,
        executable: &Executable,
        result: Result<Vec<InstructionOutput>, TransactionExecutionError>,
    ) -> (
        CostingParameters,
        TransactionFeeSummary,
        Option<TransactionFeeDetails>,
        TransactionResult,
    );

    fn finalize_receipt(
        executable: &Executable,
        costing_parameters: CostingParameters,
        fee_summary: TransactionFeeSummary,
        fee_details: Option<TransactionFeeDetails>,
        transaction_result: TransactionResult,
        resources_usage: Option<ResourcesUsage>,
    ) -> TransactionReceipt;

    fn on_pin_node(&mut self, node_id: &NodeId) -> Result<(), RuntimeError>;

    fn on_create_node<Y>(api: &mut Y, event: CreateNodeEvent) -> Result<(), RuntimeError>
        where
            Y: KernelInternalApi<Self>;

    fn on_drop_node<Y>(api: &mut Y, event: DropNodeEvent) -> Result<(), RuntimeError>
        where
            Y: KernelInternalApi<Self>;

    fn on_move_module<Y>(api: &mut Y, event: MoveModuleEvent) -> Result<(), RuntimeError>
        where
            Y: KernelInternalApi<Self>;

    fn on_open_substate<Y>(api: &mut Y, event: OpenSubstateEvent) -> Result<(), RuntimeError>
        where
            Y: KernelInternalApi<Self>;

    fn on_close_substate<Y>(api: &mut Y, event: CloseSubstateEvent) -> Result<(), RuntimeError>
        where
            Y: KernelInternalApi<Self>;

    fn on_read_substate<Y>(api: &mut Y, event: ReadSubstateEvent) -> Result<(), RuntimeError>
        where
            Y: KernelInternalApi<Self>;

    fn on_write_substate<Y>(api: &mut Y, event: WriteSubstateEvent) -> Result<(), RuntimeError>
        where
            Y: KernelInternalApi<Self>;

    fn on_set_substate(&mut self, event: SetSubstateEvent) -> Result<(), RuntimeError>;

    fn on_remove_substate(&mut self, event: RemoveSubstateEvent) -> Result<(), RuntimeError>;

    fn on_scan_keys(&mut self, event: ScanKeysEvent) -> Result<(), RuntimeError>;

    fn on_drain_substates(&mut self, event: DrainSubstatesEvent) -> Result<(), RuntimeError>;

    fn on_scan_sorted_substates(
        &mut self,
        event: ScanSortedSubstatesEvent,
    ) -> Result<(), RuntimeError>;

    fn before_invoke<Y>(
        invocation: &KernelInvocation<Self::CallFrameData>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
        where
            Y: KernelApi<Self>;

    fn after_invoke<Y>(output: &IndexedScryptoValue, api: &mut Y) -> Result<(), RuntimeError>
        where
            Y: KernelApi<Self>;

    fn on_execution_start<Y>(api: &mut Y) -> Result<(), RuntimeError>
        where
            Y: KernelApi<Self>;

    fn on_execution_finish<Y>(message: &CallFrameMessage, api: &mut Y) -> Result<(), RuntimeError>
        where
            Y: KernelApi<Self>;

    fn on_allocate_node_id<Y>(entity_type: EntityType, api: &mut Y) -> Result<(), RuntimeError>
        where
            Y: KernelApi<Self>;

    fn invoke_upstream<Y>(
        args: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: KernelApi<Self>;

    fn auto_drop<Y>(nodes: Vec<NodeId>, api: &mut Y) -> Result<(), RuntimeError>
        where
            Y: KernelApi<Self>;

    fn on_mark_substate_as_transient(
        &mut self,
        node_id: &NodeId,
        partition_number: &PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Result<(), RuntimeError>;

    fn on_substate_lock_fault<Y>(
        node_id: NodeId,
        partition_num: PartitionNumber,
        offset: &SubstateKey,
        api: &mut Y,
    ) -> Result<bool, RuntimeError>
        where
            Y: KernelApi<Self>;

    fn on_drop_node_mut<Y>(node_id: &NodeId, api: &mut Y) -> Result<(), RuntimeError>
        where
            Y: KernelApi<Self>;

    // This is technically not a kernel event, but system event, per current implementation.
    fn on_move_node<Y>(
        node_id: &NodeId,
        is_moving_down: bool,
        is_to_barrier: bool,
        destination_blueprint_id: Option<BlueprintId>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
        where
            Y: KernelApi<Self>;
}
