use super::call_frame::CallFrameUpdate;
use super::kernel_api::KernelModuleApi;
use crate::errors::RuntimeError;
use crate::kernel::actor::Actor;
use crate::system::node_init::NodeInit;
use crate::types::*;
use radix_engine_interface::api::substate_api::LockFlags;
use sbor::rust::collections::BTreeMap;

pub trait KernelModule {
    //======================
    // Kernel module setup
    //======================
    #[inline(always)]
    fn on_init<Y: KernelModuleApi<RuntimeError>>(_api: &mut Y) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_teardown<Y: KernelModuleApi<RuntimeError>>(_api: &mut Y) -> Result<(), RuntimeError> {
        Ok(())
    }

    //======================
    // Invocation events
    //
    // -> BeforeInvoke
    // -> BeforePushFrame
    //        -> ExecutionStart
    //        -> ExecutionFinish
    // -> AfterPopFrame
    // -> AfterInvoke
    //======================

    #[inline(always)]
    fn before_invoke<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _identifier: &InvocationDebugIdentifier,
        _input_size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn before_push_frame<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _callee: &Actor,
        _down_movement: &mut CallFrameUpdate,
        _args: &IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_execution_start<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _caller: &Option<Actor>,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_execution_finish<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _caller: &Option<Actor>,
        _up_movement: &CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn after_pop_frame<Y: KernelModuleApi<RuntimeError>>(_api: &mut Y) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn after_invoke<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _output_size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    //======================
    // RENode events
    //======================

    #[inline(always)]
    fn on_allocate_node_id<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _node_type: &EntityType,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn before_create_node<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _node_id: &NodeId,
        _node_init: &NodeInit,
        _node_module_init: &BTreeMap<TypedModuleId, BTreeMap<SubstateKey, IndexedScryptoValue>>,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn after_create_node<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _node_id: &NodeId,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn before_drop_node<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _node_id: &NodeId,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn after_drop_node<Y: KernelModuleApi<RuntimeError>>(_api: &mut Y) -> Result<(), RuntimeError> {
        Ok(())
    }

    //======================
    // Substate events
    //======================

    #[inline(always)]
    fn before_lock_substate<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _node_id: &NodeId,
        _module_id: &TypedModuleId,
        _offset: &SubstateKey,
        _flags: &LockFlags,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_substate_lock_fault<Y: KernelModuleApi<RuntimeError>>(
        _node_id: NodeId,
        _module_id: TypedModuleId,
        _offset: &SubstateKey,
        _api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        Ok(false)
    }

    #[inline(always)]
    fn after_lock_substate<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _lock_handle: LockHandle,
        _size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_read_substate<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _lock_handle: LockHandle,
        _size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_write_substate<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _lock_handle: LockHandle,
        _size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_drop_lock<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _lock_handle: LockHandle,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }
}
