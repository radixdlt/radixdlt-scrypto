use crate::errors::RuntimeError;
use crate::kernel::*;
use crate::system::node::{RENodeInit, RENodeModuleInit};
use radix_engine_interface::api::types::*;
use radix_engine_interface::blueprints::resource::Resource;
use sbor::rust::collections::BTreeMap;

pub trait KernelModule {
    fn on_init<Y: KernelNodeApi + KernelSubstateApi>(_api: &mut Y) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_teardown<Y: KernelNodeApi + KernelSubstateApi>(_api: &mut Y) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn before_new_frame<Y: KernelNodeApi + KernelSubstateApi + KernelActorApi<RuntimeError>>(
        _api: &mut Y,
        _actor: &ResolvedActor,
        _update: &mut CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn after_actor_run<Y: KernelNodeApi + KernelSubstateApi + KernelActorApi<RuntimeError>>(
        _api: &mut Y,
        _caller: &ResolvedActor,
        _update: &CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn before_invoke<Y: KernelNodeApi + KernelSubstateApi>(
        _api: &mut Y,
        _fn_identifier: &FnIdentifier,
        _input_size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn after_invoke<Y: KernelNodeApi + KernelSubstateApi>(
        _api: &mut Y,
        _output_size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_allocate_node_id<Y: KernelNodeApi + KernelSubstateApi>(
        _api: &mut Y,
        _node_type: &RENodeType,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn before_create_node<Y: KernelNodeApi + KernelSubstateApi>(
        _api: &mut Y,
        _node_id: &RENodeId,
        _node_init: &RENodeInit,
        _node_module_init: &BTreeMap<NodeModuleId, RENodeModuleInit>,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn after_create_node<Y: KernelNodeApi + KernelSubstateApi>(
        _api: &mut Y,
        _node_id: &RENodeId,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn before_drop_node<Y: KernelNodeApi + KernelSubstateApi>(
        _api: &mut Y,
        _node_id: &RENodeId,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn after_drop_node<Y: KernelNodeApi + KernelSubstateApi>(
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_lock_substate<Y: KernelNodeApi + KernelSubstateApi>(
        _api: &mut Y,
        _node_id: &RENodeId,
        _module_id: &NodeModuleId,
        _offset: &SubstateOffset,
        _flags: &LockFlags,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_read_substate<Y: KernelNodeApi + KernelSubstateApi>(
        _api: &mut Y,
        _lock_handle: LockHandle,
        _size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_write_substate<Y: KernelNodeApi + KernelSubstateApi>(
        _api: &mut Y,
        _lock_handle: LockHandle,
        _size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_drop_lock<Y: KernelNodeApi + KernelSubstateApi>(
        _api: &mut Y,
        _lock_handle: LockHandle,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_wasm_instantiation<Y: KernelNodeApi + KernelSubstateApi>(
        _api: &mut Y,
        _code: &[u8],
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_consume_cost_units<Y: KernelNodeApi + KernelSubstateApi>(
        _api: &mut Y,
        _units: u32,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn on_credit_cost_units<Y: KernelNodeApi + KernelSubstateApi>(
        _api: &mut Y,
        _vault_id: VaultId,
        locked_fee: Resource,
        _contingent: bool,
    ) -> Result<Resource, RuntimeError> {
        Ok(locked_fee)
    }
}
