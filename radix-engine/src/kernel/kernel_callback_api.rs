use crate::errors::*;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::KernelApi;
use crate::kernel::kernel_api::KernelInvocation;
use crate::track::interface::{NodeSubstates, StoreAccessInfo};
use crate::types::*;
use radix_engine_interface::api::field_lock_api::LockFlags;

pub trait KernelCallbackObject: Sized {
    type LockData: Default + Clone;
    type CallFrameData;

    fn on_init<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn on_teardown<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn before_drop_node<Y>(node_id: &NodeId, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn after_drop_node<Y>(api: &mut Y) -> Result<(), RuntimeError>
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
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn before_lock_substate<Y>(
        node_id: &NodeId,
        module_num: &PartitionNumber,
        substate_key: &SubstateKey,
        flags: &LockFlags,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn after_lock_substate<Y>(
        handle: LockHandle,
        size: usize,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn on_drop_lock<Y>(
        lock_handle: LockHandle,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn on_read_substate<Y>(
        lock_handle: LockHandle,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn on_write_substate<Y>(
        lock_handle: LockHandle,
        size: usize,
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

    fn on_set_substate<Y>(store_access: &StoreAccessInfo, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn on_take_substates<Y>(
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn before_invoke<Y>(
        invocation: &KernelInvocation<Self::CallFrameData>,
        input_size: usize,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn after_invoke<Y>(output_size: usize, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn before_push_frame<Y>(
        callee: &Self::CallFrameData,
        update: &mut CallFrameUpdate,
        args: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn on_execution_start<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn invoke_upstream<Y>(
        args: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelApi<Self>;

    fn on_execution_finish<Y>(update: &CallFrameUpdate, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn auto_drop<Y>(nodes: Vec<NodeId>, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn after_pop_frame<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn on_substate_lock_fault<Y>(
        node_id: NodeId,
        module_num: PartitionNumber,
        offset: &SubstateKey,
        api: &mut Y,
    ) -> Result<bool, RuntimeError>
    where
        Y: KernelApi<Self>;

    fn on_allocate_node_id<Y>(
        entity_type: Option<EntityType>,
        virtual_node: bool,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;
}
