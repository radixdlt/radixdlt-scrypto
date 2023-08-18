use crate::errors::RuntimeError;
use crate::kernel::actor::Actor;
use crate::kernel::call_frame::CallFrameMessage;
use crate::kernel::kernel_api::KernelInvocation;
use crate::kernel::kernel_api::{KernelApi, KernelInternalApi};
use crate::kernel::kernel_callback_api::{
    CloseSubstateEvent, CreateNodeEvent, DrainSubstatesEvent, DropNodeEvent, KernelCallbackObject,
    MoveModuleEvent, OpenSubstateEvent, ReadSubstateEvent, RemoveSubstateEvent, ScanKeysEvent,
    ScanSortedSubstatesEvent, SetSubstateEvent, WriteSubstateEvent,
};
use crate::types::*;

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
    //        -> ExecutionStart
    //        -> ExecutionFinish
    // -> AfterInvoke
    //======================

    #[inline(always)]
    fn before_invoke<Y: KernelApi<M>>(
        _api: &mut Y,
        _invocation: &KernelInvocation<Actor>,
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
        _message: &CallFrameMessage,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn after_invoke<Y: KernelApi<M>>(
        _api: &mut Y,
        _output: &IndexedScryptoValue,
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
    fn on_drop_node<Y: KernelInternalApi<M>>(
        _api: &mut Y,
        _event: &DropNodeEvent,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    //======================
    // Substate events
    //======================
    #[inline(always)]
    fn on_mark_substate_as_transient(_system: &mut M, _node_id: &NodeId, _partition_number: &PartitionNumber, _substate_key: &SubstateKey) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_open_substate<Y: KernelInternalApi<M>>(
        _api: &mut Y,
        _event: &OpenSubstateEvent,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_read_substate<Y: KernelInternalApi<M>>(
        _api: &mut Y,
        _event: &ReadSubstateEvent,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_write_substate<Y: KernelInternalApi<M>>(
        _api: &mut Y,
        _event: &WriteSubstateEvent,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_close_substate<Y: KernelInternalApi<M>>(
        _api: &mut Y,
        _event: &CloseSubstateEvent,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_set_substate(_system: &mut M, _event: &SetSubstateEvent) -> Result<(), RuntimeError> {
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
    fn on_drain_substates(
        _system: &mut M,
        _event: &DrainSubstatesEvent,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_scan_sorted_substates(
        _system: &mut M,
        _event: &ScanSortedSubstatesEvent,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }
}
