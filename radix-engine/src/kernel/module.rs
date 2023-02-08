use crate::errors::RuntimeError;
use crate::kernel::*;
use crate::system::node::{RENodeInit, RENodeModuleInit};
use radix_engine_interface::api::types::*;
use radix_engine_interface::blueprints::resource::Resource;
use sbor::rust::collections::BTreeMap;

pub trait KernelModule {
    //======================
    // Kernel module events
    //======================

    fn on_init<Y: KernelModuleApi<RuntimeError>>(_api: &mut Y) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_teardown<Y: KernelModuleApi<RuntimeError>>(_api: &mut Y) -> Result<(), RuntimeError> {
        Ok(())
    }

    //======================
    // Invocation events:
    //
    //  BeforeInvoke -> BeforeNewFrame -> AfterActorRun -> AfterInvoke
    //======================

    fn before_invoke<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _fn_identifier: &FnIdentifier,
        _input_size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn before_new_frame<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _actor: &ResolvedActor,
        _update: &mut CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn after_execute<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _caller: &ResolvedActor,
        _update: &CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn after_invoke<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _output_size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_allocate_node_id<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _node_type: &RENodeType,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    //======================
    // RENode events
    //======================

    fn before_create_node<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _node_id: &RENodeId,
        _node_init: &RENodeInit,
        _node_module_init: &BTreeMap<NodeModuleId, RENodeModuleInit>,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn after_create_node<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _node_id: &RENodeId,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn before_drop_node<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _node_id: &RENodeId,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn after_drop_node<Y: KernelModuleApi<RuntimeError>>(_api: &mut Y) -> Result<(), RuntimeError> {
        Ok(())
    }

    //======================
    // Substate events
    //======================

    fn on_lock_substate<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _node_id: &RENodeId,
        _module_id: &NodeModuleId,
        _offset: &SubstateOffset,
        _flags: &LockFlags,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_read_substate<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _lock_handle: LockHandle,
        _size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_write_substate<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _lock_handle: LockHandle,
        _size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_drop_lock<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _lock_handle: LockHandle,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    //======================
    // WASM interpreter events
    //======================

    fn on_wasm_instantiation<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _code: &[u8],
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    //======================
    // Other events
    //======================

    fn on_consume_cost_units<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _units: u32,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_update_instruction_index<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _new_index: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_credit_cost_units<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _vault_id: VaultId,
        locked_fee: Resource,
        _contingent: bool,
    ) -> Result<Resource, RuntimeError> {
        Ok(locked_fee)
    }
}
