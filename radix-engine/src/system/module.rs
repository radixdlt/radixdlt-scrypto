use crate::errors::RuntimeError;
use crate::kernel::actor::Actor;
use crate::kernel::call_frame::Message;
use crate::kernel::kernel_api::{KernelApi, KernelInternalApi};
use crate::kernel::kernel_api::KernelInvocation;
use crate::kernel::kernel_callback_api::{CreateNodeEvent, DrainSubstatesEvent, KernelCallbackObject, MoveModuleEvent, OpenSubstateEvent, RemoveSubstateEvent, ScanKeysEvent, ScanSortedSubstatesEvent, SetSubstateEvent};
use crate::track::interface::{NodeSubstates, StoreAccess};
use crate::types::*;
use radix_engine_interface::api::field_api::LockFlags;

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
        _message: &Message,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn after_pop_frame<Y: KernelApi<M>>(
        _api: &mut Y,
        _dropped_actor: &Actor,
        _message: &Message,
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
    fn on_create_node<Y: KernelInternalApi<M>>(
        _api: &mut Y,
        _event: &CreateNodeEvent,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_move_module<Y: KernelInternalApi<M>>(
        _api: &mut Y,
        _event: &MoveModuleEvent,
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
    fn on_open_substate<Y: KernelInternalApi<M>>(
        _api: &mut Y,
        _event: &OpenSubstateEvent,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn after_open_substate<Y: KernelApi<M>>(
        _api: &mut Y,
        _lock_handle: LockHandle,
        _node_id: &NodeId,
        _size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_read_substate<Y: KernelApi<M>>(
        _api: &mut Y,
        _lock_handle: LockHandle,
        _value_size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_write_substate<Y: KernelApi<M>>(
        _api: &mut Y,
        _lock_handle: LockHandle,
        _value_size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_close_substate<Y: KernelApi<M>>(
        _api: &mut Y,
        _lock_handle: LockHandle,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_set_substate(
        _system: &mut M,
        _event: &SetSubstateEvent,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_remove_substate(
        _system: &mut M,
        _event: &RemoveSubstateEvent,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_scan_keys(_system: &mut M, _event: &ScanKeysEvent) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_drain_substates(_system: &mut M, _event: &DrainSubstatesEvent) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_scan_sorted_substates(_system: &mut M, _event: &ScanSortedSubstatesEvent) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_store_access(_store_access: &StoreAccess, _system: &mut M) -> Result<(), RuntimeError> {
        Ok(())
    }
}
