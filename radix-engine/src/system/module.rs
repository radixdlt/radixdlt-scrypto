use crate::errors::RuntimeError;
use crate::kernel::actor::Actor;
use crate::kernel::call_frame::Message;
use crate::kernel::kernel_api::KernelApi;
use crate::kernel::kernel_api::KernelInvocation;
use crate::kernel::kernel_callback_api::KernelCallbackObject;
use crate::track::interface::{NodeSubstates, StoreAccessInfo};
use crate::types::*;
use radix_engine_interface::api::field_lock_api::LockFlags;

pub trait SystemModule<M: KernelCallbackObject> {
    //======================
    // System module setup
    //======================
    #[inline(always)]
    fn on_init<Y: KernelApi<M>>(_api: &mut Y) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_teardown<Y: KernelApi<M>>(_api: &mut Y) -> Result<(), RuntimeError> {
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
    fn before_invoke<Y: KernelApi<M>>(
        _api: &mut Y,
        _invocation: &KernelInvocation,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn before_push_frame<Y: KernelApi<M>>(
        _api: &mut Y,
        _callee: &Actor,
        _message: &mut Message,
        _args: &IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_execution_start<Y: KernelApi<M>>(_api: &mut Y) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_execution_finish<Y: KernelApi<M>>(
        _api: &mut Y,
        _up_movement: &Message,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn after_pop_frame<Y: KernelApi<M>>(
        _api: &mut Y,
        _dropped_actor: &Actor,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn after_invoke<Y: KernelApi<M>>(
        _api: &mut Y,
        _output_size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    //======================
    // RENode events
    //======================

    #[inline(always)]
    fn on_allocate_node_id<Y: KernelApi<M>>(
        _api: &mut Y,
        _entity_type: EntityType,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn before_create_node<Y: KernelApi<M>>(
        _api: &mut Y,
        _node_id: &NodeId,
        _node_substates: &NodeSubstates,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn after_create_node<Y: KernelApi<M>>(
        _api: &mut Y,
        _node_id: &NodeId,
        _total_substate_size: usize,
        _store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn after_move_modules<Y: KernelApi<M>>(
        _api: &mut Y,
        _src_node_id: &NodeId,
        _dest_node_id: &NodeId,
        _store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn before_drop_node<Y: KernelApi<M>>(
        _api: &mut Y,
        _node_id: &NodeId,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn after_drop_node<Y: KernelApi<M>>(
        _api: &mut Y,
        _total_substate_size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    //======================
    // Substate events
    //======================

    #[inline(always)]
    fn before_open_substate<Y: KernelApi<M>>(
        _api: &mut Y,
        _node_id: &NodeId,
        _partition_num: &PartitionNumber,
        _offset: &SubstateKey,
        _flags: &LockFlags,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn after_open_substate<Y: KernelApi<M>>(
        _api: &mut Y,
        _lock_handle: LockHandle,
        _node_id: &NodeId,
        _store_access: &StoreAccessInfo,
        _size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_read_substate<Y: KernelApi<M>>(
        _api: &mut Y,
        _lock_handle: LockHandle,
        _value_size: usize,
        _store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_write_substate<Y: KernelApi<M>>(
        _api: &mut Y,
        _lock_handle: LockHandle,
        _value_size: usize,
        _store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_close_substate<Y: KernelApi<M>>(
        _api: &mut Y,
        _lock_handle: LockHandle,
        _store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_scan_substate<Y: KernelApi<M>>(
        _api: &mut Y,
        _store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_set_substate<Y: KernelApi<M>>(
        _api: &mut Y,
        _value_size: usize,
        _store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_take_substates<Y: KernelApi<M>>(
        _api: &mut Y,
        _store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }
}
