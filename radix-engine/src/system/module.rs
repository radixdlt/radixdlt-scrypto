use crate::errors::RuntimeError;
use crate::internal_prelude::*;
use crate::kernel::call_frame::CallFrameMessage;
use crate::kernel::kernel_api::*;
use crate::kernel::kernel_callback_api::*;
use crate::system::actor::Actor;

use super::system_callback::*;
use super::system_callback_api::*;

/// We use a zero-cost `SystemModuleApiImpl` wrapper instead of just implementing
/// on `K` for a few reasons:
/// * Separation of APIs - we avoid exposing a `SystemModuleApi` directly if someone
///   happens to have a SystemBasedKernelApi, as that might be confusing.
/// * Trait coherence - even if `SystemModuleApi` were moved to a different crate,
///   this would still work
pub struct SystemModuleApiImpl<'a, K: KernelInternalApi + ?Sized> {
    api: &'a mut K,
}

impl<'a, K: KernelInternalApi + ?Sized> SystemModuleApiImpl<'a, K> {
    #[inline]
    pub fn new(api: &'a mut K) -> Self {
        Self { api }
    }

    #[inline]
    pub fn api_ref(&self) -> &K {
        &self.api
    }

    #[inline]
    pub fn api(&mut self) -> &mut K {
        &mut self.api
    }
}

pub trait SystemModuleApi {
    type SystemCallback: SystemCallbackObject;

    fn system(&mut self) -> &mut System<Self::SystemCallback>;

    fn system_state(&mut self) -> SystemState<'_, System<Self::SystemCallback>>;

    fn current_stack_depth_uncosted(&self) -> usize;

    fn current_stack_id_uncosted(&self) -> usize;
}

impl<'a, V: SystemCallbackObject, K: KernelInternalApi<System = System<V>> + ?Sized> SystemModuleApi
    for SystemModuleApiImpl<'a, K>
{
    type SystemCallback = V;

    fn system(&mut self) -> &mut K::System {
        self.api.kernel_get_system()
    }

    fn system_state(&mut self) -> SystemState<'_, K::System> {
        self.api.kernel_get_system_state()
    }

    fn current_stack_depth_uncosted(&self) -> usize {
        self.api.kernel_get_current_stack_depth_uncosted()
    }

    fn current_stack_id_uncosted(&self) -> usize {
        self.api.kernel_get_current_stack_id_uncosted()
    }
}

pub trait ResolvableSystemModule {
    fn resolve_from_system(system: &mut impl HasModules) -> &mut Self;
}

pub trait SystemModuleApiFor<M: ResolvableSystemModule + ?Sized>: SystemModuleApi {
    fn module(&mut self) -> &mut M {
        M::resolve_from_system(self.system())
    }
}

impl<
        'a,
        V: SystemCallbackObject,
        K: KernelInternalApi<System = System<V>> + ?Sized,
        M: ResolvableSystemModule + ?Sized,
    > SystemModuleApiFor<M> for SystemModuleApiImpl<'a, K>
{
}

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

pub trait PrivilegedSystemModule {
    #[inline(always)]
    fn privileged_before_invoke(
        _api: &mut impl SystemBasedKernelApi,
        _invocation: &KernelInvocation<Actor>,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }
}

pub trait SystemModule<ModuleApi: SystemModuleApiFor<Self>>:
    InitSystemModule + ResolvableSystemModule + PrivilegedSystemModule
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
        _api: &mut ModuleApi,
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

    fn on_get_stack_id(_api: &mut ModuleApi) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_switch_stack(_api: &mut ModuleApi) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_send_to_stack(_api: &mut ModuleApi, _data_len: usize) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_set_call_frame_data(_api: &mut ModuleApi, _data_len: usize) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_get_owned_nodes(_api: &mut ModuleApi) -> Result<(), RuntimeError> {
        Ok(())
    }
}
