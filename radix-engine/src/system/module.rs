use crate::errors::RuntimeError;
use crate::kernel::actor::Actor;
use crate::kernel::call_frame::Message;
use crate::kernel::kernel_api::KernelApi;
use crate::kernel::kernel_api::KernelInvocation;
use crate::kernel::kernel_callback_api::KernelCallbackObject;
use crate::track::interface::NodeSubstates;
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
        _node_type: Option<EntityType>,
        _virtual_node: bool,
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
    fn after_drop_node<Y: KernelApi<M>>(_api: &mut Y) -> Result<(), RuntimeError> {
        Ok(())
    }

    //======================
    // Substate events
    //======================

    #[inline(always)]
    fn before_lock_substate<Y: KernelApi<M>>(
        _api: &mut Y,
        _node_id: &NodeId,
        _partition_num: &PartitionNumber,
        _offset: &SubstateKey,
        _flags: &LockFlags,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn after_lock_substate<Y: KernelApi<M>>(
        _api: &mut Y,
        _lock_handle: LockHandle,
        _first_lock_from_db: bool,
        _size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_read_substate<Y: KernelApi<M>>(
        _api: &mut Y,
        _lock_handle: LockHandle,
        _size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_write_substate<Y: KernelApi<M>>(
        _api: &mut Y,
        _lock_handle: LockHandle,
        _size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_drop_lock<Y: KernelApi<M>>(
        _api: &mut Y,
        _lock_handle: LockHandle,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }
}
