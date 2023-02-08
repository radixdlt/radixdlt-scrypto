use super::KernelModule;
use crate::errors::*;
use crate::kernel::*;
use crate::system::kernel_modules::auth::auth_module::AuthModule;
use crate::system::kernel_modules::costing::CostingModule;
use crate::system::kernel_modules::execution_trace::ExecutionTraceModule;
use crate::system::kernel_modules::kernel_debug::KernelDebugModule;
use crate::system::kernel_modules::logger::LoggerModule;
use crate::system::kernel_modules::node_move::NodeMoveModule;
use crate::system::kernel_modules::transaction_runtime::TransactionRuntimeModule;
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use radix_engine_interface::api::types::FnIdentifier;
use radix_engine_interface::api::types::LockHandle;
use radix_engine_interface::api::types::NodeModuleId;
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::api::types::RENodeType;
use radix_engine_interface::api::types::SubstateOffset;
use radix_engine_interface::api::types::VaultId;
use radix_engine_interface::blueprints::resource::Resource;
use sbor::rust::collections::BTreeMap;

pub struct KernelModuleMixer;

//====================================================================
// NOTE: Modules are applied in the opposite order of initialization!
//====================================================================

impl KernelModule for KernelModuleMixer {
    fn on_init<Y: KernelNodeApi + KernelSubstateApi>(api: &mut Y) -> Result<(), RuntimeError> {
        // Enable execution trace
        ExecutionTraceModule::on_init(api)?;

        // Enable transaction runtime
        TransactionRuntimeModule::on_init(api)?;

        // Enable logger
        LoggerModule::on_init(api)?;

        // Enable auth
        AuthModule::on_init(api)?;

        // Enable node move
        NodeMoveModule::on_init(api)?;

        // Enable costing
        CostingModule::on_init(api)?;

        // Enable debug
        KernelDebugModule::on_init(api)?;

        Ok(())
    }

    fn on_teardown<Y: KernelNodeApi + KernelSubstateApi>(api: &mut Y) -> Result<(), RuntimeError> {
        KernelDebugModule::on_teardown(api)?;
        CostingModule::on_teardown(api)?;
        NodeMoveModule::on_teardown(api)?;
        AuthModule::on_teardown(api)?;
        LoggerModule::on_teardown(api)?;
        TransactionRuntimeModule::on_teardown(api)?;
        ExecutionTraceModule::on_teardown(api)?;
        Ok(())
    }

    fn before_create_frame<Y: KernelNodeApi + KernelSubstateApi + KernelActorApi<RuntimeError>>(
        api: &mut Y,
        actor: &ResolvedActor,
        update: &mut CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        KernelDebugModule::before_create_frame(api, actor, update)?;
        CostingModule::before_create_frame(api, actor, update)?;
        NodeMoveModule::before_create_frame(api, actor, update)?;
        AuthModule::before_create_frame(api, actor, update)?;
        LoggerModule::before_create_frame(api, actor, update)?;
        TransactionRuntimeModule::before_create_frame(api, actor, update)?;
        ExecutionTraceModule::before_create_frame(api, actor, update)?;
        Ok(())
    }





    fn after_actor_run<Y: KernelNodeApi + KernelSubstateApi + KernelActorApi<RuntimeError>>(
        api: &mut Y,
        caller: &ResolvedActor,
        update: &CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        KernelDebugModule::after_actor_run(api, caller, update)?;
        CostingModule::after_actor_run(api, caller, update)?;
        NodeMoveModule::after_actor_run(api, caller, update)?;
        AuthModule::after_actor_run(api, caller, update)?;
        LoggerModule::after_actor_run(api, caller, update)?;
        TransactionRuntimeModule::after_actor_run(api, caller, update)?;
        ExecutionTraceModule::after_actor_run(api, caller, update)?;
        Ok(())
    }

    fn pre_kernel_invoke<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        fn_identifier: &FnIdentifier,
        input_size: usize,
    ) -> Result<(), RuntimeError> {
        KernelDebugModule::pre_kernel_invoke(api, fn_identifier, input_size)?;
        CostingModule::pre_kernel_invoke(api, fn_identifier, input_size)?;
        NodeMoveModule::pre_kernel_invoke(api, fn_identifier, input_size)?;
        AuthModule::pre_kernel_invoke(api, fn_identifier, input_size)?;
        LoggerModule::pre_kernel_invoke(api, fn_identifier, input_size)?;
        TransactionRuntimeModule::pre_kernel_invoke(api, fn_identifier, input_size)?;
        ExecutionTraceModule::pre_kernel_invoke(api, fn_identifier, input_size)?;
        Ok(())
    }

    fn post_kernel_invoke<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        output_size: usize,
    ) -> Result<(), RuntimeError> {
        KernelDebugModule::post_kernel_invoke(api, output_size)?;
        CostingModule::post_kernel_invoke(api, output_size)?;
        NodeMoveModule::post_kernel_invoke(api, output_size)?;
        AuthModule::post_kernel_invoke(api, output_size)?;
        LoggerModule::post_kernel_invoke(api, output_size)?;
        TransactionRuntimeModule::post_kernel_invoke(api, output_size)?;
        ExecutionTraceModule::post_kernel_invoke(api, output_size)?;
        Ok(())
    }

    fn on_allocate_node_id<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        node_type: &RENodeType,
    ) -> Result<(), RuntimeError> {
        KernelDebugModule::on_allocate_node_id(api, node_type)?;
        CostingModule::on_allocate_node_id(api, node_type)?;
        NodeMoveModule::on_allocate_node_id(api, node_type)?;
        AuthModule::on_allocate_node_id(api, node_type)?;
        LoggerModule::on_allocate_node_id(api, node_type)?;
        TransactionRuntimeModule::on_allocate_node_id(api, node_type)?;
        ExecutionTraceModule::on_allocate_node_id(api, node_type)?;
        Ok(())
    }

    fn pre_create_node<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        node_id: &RENodeId,
        node_init: &RENodeInit,
        node_module_init: &BTreeMap<NodeModuleId, RENodeModuleInit>,
    ) -> Result<(), RuntimeError> {
        KernelDebugModule::pre_create_node(api, node_id, node_init, node_module_init)?;
        CostingModule::pre_create_node(api, node_id, node_init, node_module_init)?;
        NodeMoveModule::pre_create_node(api, node_id, node_init, node_module_init)?;
        AuthModule::pre_create_node(api, node_id, node_init, node_module_init)?;
        LoggerModule::pre_create_node(api, node_id, node_init, node_module_init)?;
        TransactionRuntimeModule::pre_create_node(api, node_id, node_init, node_module_init)?;
        ExecutionTraceModule::pre_create_node(api, node_id, node_init, node_module_init)?;
        Ok(())
    }

    fn post_create_node<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        node_id: &RENodeId,
    ) -> Result<(), RuntimeError> {
        KernelDebugModule::post_create_node(api, node_id)?;
        CostingModule::post_create_node(api, node_id)?;
        NodeMoveModule::post_create_node(api, node_id)?;
        AuthModule::post_create_node(api, node_id)?;
        LoggerModule::post_create_node(api, node_id)?;
        TransactionRuntimeModule::post_create_node(api, node_id)?;
        ExecutionTraceModule::post_create_node(api, node_id)?;
        Ok(())
    }

    fn pre_drop_node<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        node_id: &RENodeId,
    ) -> Result<(), RuntimeError> {
        KernelDebugModule::pre_drop_node(api, node_id)?;
        CostingModule::pre_drop_node(api, node_id)?;
        NodeMoveModule::pre_drop_node(api, node_id)?;
        AuthModule::pre_drop_node(api, node_id)?;
        LoggerModule::pre_drop_node(api, node_id)?;
        TransactionRuntimeModule::pre_drop_node(api, node_id)?;
        ExecutionTraceModule::pre_drop_node(api, node_id)?;
        Ok(())
    }

    fn post_drop_node<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        KernelDebugModule::post_drop_node(api)?;
        CostingModule::post_drop_node(api)?;
        NodeMoveModule::post_drop_node(api)?;
        AuthModule::post_drop_node(api)?;
        LoggerModule::post_drop_node(api)?;
        TransactionRuntimeModule::post_drop_node(api)?;
        ExecutionTraceModule::post_drop_node(api)?;
        Ok(())
    }

    fn on_lock_substate<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        node_id: &RENodeId,
        module_id: &NodeModuleId,
        offset: &SubstateOffset,
        flags: &LockFlags,
    ) -> Result<(), RuntimeError> {
        KernelDebugModule::on_lock_substate(api, node_id, module_id, offset, flags)?;
        CostingModule::on_lock_substate(api, node_id, module_id, offset, flags)?;
        NodeMoveModule::on_lock_substate(api, node_id, module_id, offset, flags)?;
        AuthModule::on_lock_substate(api, node_id, module_id, offset, flags)?;
        LoggerModule::on_lock_substate(api, node_id, module_id, offset, flags)?;
        TransactionRuntimeModule::on_lock_substate(api, node_id, module_id, offset, flags)?;
        ExecutionTraceModule::on_lock_substate(api, node_id, module_id, offset, flags)?;
        Ok(())
    }

    fn on_read_substate<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        lock_handle: LockHandle,
        size: usize,
    ) -> Result<(), RuntimeError> {
        KernelDebugModule::on_read_substate(api, lock_handle, size)?;
        CostingModule::on_read_substate(api, lock_handle, size)?;
        NodeMoveModule::on_read_substate(api, lock_handle, size)?;
        AuthModule::on_read_substate(api, lock_handle, size)?;
        LoggerModule::on_read_substate(api, lock_handle, size)?;
        TransactionRuntimeModule::on_read_substate(api, lock_handle, size)?;
        ExecutionTraceModule::on_read_substate(api, lock_handle, size)?;
        Ok(())
    }

    fn on_write_substate<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        lock_handle: LockHandle,
        size: usize,
    ) -> Result<(), RuntimeError> {
        KernelDebugModule::on_write_substate(api, lock_handle, size)?;
        CostingModule::on_write_substate(api, lock_handle, size)?;
        NodeMoveModule::on_write_substate(api, lock_handle, size)?;
        AuthModule::on_write_substate(api, lock_handle, size)?;
        LoggerModule::on_write_substate(api, lock_handle, size)?;
        TransactionRuntimeModule::on_write_substate(api, lock_handle, size)?;
        ExecutionTraceModule::on_write_substate(api, lock_handle, size)?;
        Ok(())
    }

    fn on_drop_lock<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        lock_handle: LockHandle,
    ) -> Result<(), RuntimeError> {
        KernelDebugModule::on_drop_lock(api, lock_handle)?;
        CostingModule::on_drop_lock(api, lock_handle)?;
        NodeMoveModule::on_drop_lock(api, lock_handle)?;
        AuthModule::on_drop_lock(api, lock_handle)?;
        LoggerModule::on_drop_lock(api, lock_handle)?;
        TransactionRuntimeModule::on_drop_lock(api, lock_handle)?;
        ExecutionTraceModule::on_drop_lock(api, lock_handle)?;
        Ok(())
    }

    fn on_wasm_instantiation<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        code: &[u8],
    ) -> Result<(), RuntimeError> {
        KernelDebugModule::on_wasm_instantiation(api, code)?;
        CostingModule::on_wasm_instantiation(api, code)?;
        NodeMoveModule::on_wasm_instantiation(api, code)?;
        AuthModule::on_wasm_instantiation(api, code)?;
        LoggerModule::on_wasm_instantiation(api, code)?;
        TransactionRuntimeModule::on_wasm_instantiation(api, code)?;
        ExecutionTraceModule::on_wasm_instantiation(api, code)?;
        Ok(())
    }

    fn on_consume_cost_units<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        units: u32,
    ) -> Result<(), RuntimeError> {
        KernelDebugModule::on_consume_cost_units(api, units)?;
        CostingModule::on_consume_cost_units(api, units)?;
        NodeMoveModule::on_consume_cost_units(api, units)?;
        AuthModule::on_consume_cost_units(api, units)?;
        LoggerModule::on_consume_cost_units(api, units)?;
        TransactionRuntimeModule::on_consume_cost_units(api, units)?;
        ExecutionTraceModule::on_consume_cost_units(api, units)?;
        Ok(())
    }

    fn on_credit_cost_units<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        vault_id: VaultId,
        fee: Resource,
        contingent: bool,
    ) -> Result<Resource, RuntimeError> {
        fee = KernelDebugModule::on_credit_cost_units(api, vault_id, fee, contingent)?;
        fee = CostingModule::on_credit_cost_units(api, vault_id, fee, contingent)?;
        fee = NodeMoveModule::on_credit_cost_units(api, vault_id, fee, contingent)?;
        fee = AuthModule::on_credit_cost_units(api, vault_id, fee, contingent)?;
        fee = LoggerModule::on_credit_cost_units(api, vault_id, fee, contingent)?;
        fee = TransactionRuntimeModule::on_credit_cost_units(api, vault_id, fee, contingent)?;
        fee = ExecutionTraceModule::on_credit_cost_units(api, vault_id, fee, contingent)?;
        Ok(fee)
    }
}
