use crate::errors::RuntimeError;
use crate::internal_prelude::*;
use crate::kernel::call_frame::CallFrameMessage;
use crate::kernel::kernel_api::KernelInvocation;
use crate::kernel::kernel_callback_api::*;
use crate::system::actor::Actor;

use super::system_callback::*;

pub trait InitSystemModule {
    //======================
    // System module setup
    //======================
    #[inline(always)]
    fn init(&mut self) -> Result<(), BootloadingError> {
        Ok(())
    }

    #[inline(always)]
    fn on_teardown(&mut self) -> Result<(), RuntimeError> {
        Ok(())
    }
}

pub trait SystemModule<ModuleApi: SystemModuleApiFor<Self>>:
    InitSystemModule + ResolvableSystemModule
{
    //======================
    // Invocation events
    //
    // -> BeforeInvoke
    //        -> ExecutionStart
    //        -> ExecutionFinish
    // -> AfterInvoke
    //======================

    #[inline(always)]
    fn before_invoke(
        _api: &mut ModuleApi, // For charge_package_royalty in the Costing Module
        _invocation: &KernelInvocation<Actor>,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_execution_start(_api: &mut ModuleApi) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_execution_finish(
        _api: &mut ModuleApi,
        _message: &CallFrameMessage,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn after_invoke(
        _api: &mut ModuleApi,
        _output: &IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    //======================
    // RENode events
    //======================

    #[inline(always)]
    fn on_pin_node(_api: &mut ModuleApi, _node_id: &NodeId) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_allocate_node_id(
        _api: &mut ModuleApi,
        _entity_type: EntityType,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_create_node(_api: &mut ModuleApi, _event: &CreateNodeEvent) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_move_module(_api: &mut ModuleApi, _event: &MoveModuleEvent) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_drop_node(_api: &mut ModuleApi, _event: &DropNodeEvent) -> Result<(), RuntimeError> {
        Ok(())
    }

    //======================
    // Substate events
    //======================
    #[inline(always)]
    fn on_mark_substate_as_transient(
        _api: &mut ModuleApi,
        _node_id: &NodeId,
        _partition_number: &PartitionNumber,
        _substate_key: &SubstateKey,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_open_substate(
        _api: &mut ModuleApi,
        _event: &OpenSubstateEvent,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_read_substate(
        _api: &mut ModuleApi,
        _event: &ReadSubstateEvent,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_write_substate(
        _api: &mut ModuleApi,
        _event: &WriteSubstateEvent,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_close_substate(
        _api: &mut ModuleApi,
        _event: &CloseSubstateEvent,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_set_substate(
        _api: &mut ModuleApi,
        _event: &SetSubstateEvent,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_remove_substate(
        _api: &mut ModuleApi,
        _event: &RemoveSubstateEvent,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_scan_keys(_api: &mut ModuleApi, _event: &ScanKeysEvent) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_drain_substates(
        _api: &mut ModuleApi,
        _event: &DrainSubstatesEvent,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_scan_sorted_substates(
        _api: &mut ModuleApi,
        _event: &ScanSortedSubstatesEvent,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }
}
