use crate::errors::*;
use crate::kernel::actor::Actor;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::KernelApi;
use crate::kernel::kernel_api::KernelInvocation;
use crate::types::*;
use radix_engine_interface::api::substate_lock_api::LockFlags;
use radix_engine_stores::interface::NodeSubstates;

pub trait KernelCallbackObject: Sized {
    type Invocation: Debug;
    type LockData: Default + Clone;

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

    fn after_create_node<Y>(node_id: &NodeId, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn before_lock_substate<Y>(
        node_id: &NodeId,
        module_num: &ModuleNumber,
        substate_key: &SubstateKey,
        flags: &LockFlags,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn after_lock_substate<Y>(
        handle: LockHandle,
        size: usize,
        first_lock_from_db: bool,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn on_drop_lock<Y>(lock_handle: LockHandle, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn on_read_substate<Y>(
        lock_handle: LockHandle,
        size: usize,
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

    fn before_invoke<Y>(
        identifier: &KernelInvocation<Self::Invocation>,
        input_size: usize,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn after_invoke<Y>(output_size: usize, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn before_push_frame<Y>(
        callee: &Actor,
        update: &mut CallFrameUpdate,
        args: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn on_execution_start<Y>(caller: &Option<Actor>, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>;

    fn invoke_upstream<Y>(
        invocation: Self::Invocation,
        args: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelApi<Self>;

    fn on_execution_finish<Y>(
        caller: &Option<Actor>,
        update: &CallFrameUpdate,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
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
        module_num: ModuleNumber,
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
