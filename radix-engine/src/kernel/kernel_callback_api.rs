use super::call_frame::CallFrameMessage;
use crate::errors::*;
use crate::kernel::kernel_api::KernelInvocation;
use crate::kernel::kernel_api::{KernelApi, KernelInternalApi};
use crate::kernel::substate_io::SubstateDevice;
use crate::track::interface::{NodeSubstates, StoreAccess};
use crate::types::*;
use radix_engine_interface::api::field_api::LockFlags;

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
    StoreAccess(&'a StoreAccess),
    End(&'a NodeId),
}

#[derive(Debug)]
pub enum DropNodeEvent<'a> {
    Start(&'a NodeId),
    StoreAccess(&'a StoreAccess),
    End(&'a NodeId, &'a NodeSubstates),
}

#[derive(Debug)]
pub enum MoveModuleEvent<'a> {
    StoreAccess(&'a StoreAccess),
}

#[derive(Debug)]
pub enum OpenSubstateEvent<'a> {
    Start {
        node_id: &'a NodeId,
        partition_num: &'a PartitionNumber,
        substate_key: &'a SubstateKey,
        flags: &'a LockFlags,
    },
    StoreAccess(&'a StoreAccess),
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
    StoreAccess(&'a StoreAccess),
}

#[derive(Debug)]
pub enum WriteSubstateEvent<'a> {
    Start {
        handle: SubstateHandle,
        value: &'a IndexedScryptoValue,
    },
    StoreAccess(&'a StoreAccess),
}

#[derive(Debug)]
pub enum CloseSubstateEvent {
    End(SubstateHandle),
}

#[derive(Debug)]
pub enum SetSubstateEvent<'a> {
    Start(
        &'a NodeId,
        &'a PartitionNumber,
        &'a SubstateKey,
        &'a IndexedScryptoValue,
    ),
    StoreAccess(&'a StoreAccess),
}

#[derive(Debug)]
pub enum RemoveSubstateEvent<'a> {
    Start(&'a NodeId, &'a PartitionNumber, &'a SubstateKey),
    StoreAccess(&'a StoreAccess),
}

#[derive(Debug)]
pub enum ScanKeysEvent<'a> {
    Start,
    StoreAccess(&'a StoreAccess),
}

#[derive(Debug)]
pub enum DrainSubstatesEvent<'a> {
    Start(u32),
    StoreAccess(&'a StoreAccess),
}

#[derive(Debug)]
pub enum ScanSortedSubstatesEvent<'a> {
    Start,
    StoreAccess(&'a StoreAccess),
}

pub trait KernelCallbackObject: Sized {
    type LockData: Default + Clone;
    type CallFrameData: CallFrameReferences;

    fn on_init<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn on_teardown<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

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
