use crate::errors::*;
use crate::kernel::kernel_api::KernelApi;
use crate::kernel::kernel_api::KernelInvocation;
use crate::track::interface::{NodeSubstates, StoreAccessInfo};
use crate::types::*;
use radix_engine_interface::api::field_api::LockFlags;

use super::call_frame::Message;

pub trait CallFrameReferences {
    fn global_references(&self) -> Vec<GlobalAddress>;
    fn direct_access_references(&self) -> Vec<NodeId>;
    fn transient_references(&self) -> Vec<NodeId>;

    fn len(&self) -> usize;
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

    fn before_drop_node<Y>(node_id: &NodeId, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn after_drop_node<Y>(api: &mut Y, total_substate_size: usize) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn before_create_node<Y>(
        node_id: &NodeId,
        node_substates: &NodeSubstates,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn after_create_node<Y>(
        node_id: &NodeId,
        total_substate_size: usize,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn after_move_modules<Y>(
        src_node_id: &NodeId,
        dest_node_id: &NodeId,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn before_open_substate<Y>(
        node_id: &NodeId,
        partition_num: &PartitionNumber,
        substate_key: &SubstateKey,
        flags: &LockFlags,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn after_open_substate<Y>(
        handle: LockHandle,
        node_id: &NodeId,
        size: usize,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn on_close_substate<Y>(
        lock_handle: LockHandle,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn on_read_substate<Y>(
        lock_handle: LockHandle,
        value_size: usize,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn on_write_substate<Y>(
        lock_handle: LockHandle,
        value_size: usize,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn on_scan_substates<Y>(
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn on_set_substate<Y>(
        value_size: usize,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn on_drain_substates<Y>(
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn before_invoke<Y>(
        invocation: &KernelInvocation<Self::CallFrameData>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    // TODO: Remove
    fn on_execution_start<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    // TODO: Remove
    fn on_execution_finish<Y>(message: &Message, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn after_invoke<Y>(output: &IndexedScryptoValue, api: &mut Y) -> Result<(), RuntimeError>
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

    fn on_substate_lock_fault<Y>(
        node_id: NodeId,
        partition_num: PartitionNumber,
        offset: &SubstateKey,
        api: &mut Y,
    ) -> Result<bool, RuntimeError>
    where
        Y: KernelApi<Self>;

    fn on_drop_node<Y>(node_id: &NodeId, api: &mut Y) -> Result<(), RuntimeError>
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
