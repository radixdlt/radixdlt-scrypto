use super::KernelModule;
use crate::errors::*;
use crate::kernel::*;
use crate::system::kernel_modules::auth::AuthModule;
use crate::system::kernel_modules::costing::CostingModule;
use crate::system::kernel_modules::costing::FeeTable;
use crate::system::kernel_modules::costing::SystemLoanFeeReserve;
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
use radix_engine_interface::crypto::Hash;
use sbor::rust::collections::BTreeMap;
use transaction::model::AuthZoneParams;

pub struct KernelModuleMixer {
    /* flags */
    pub kernel_debug_enabled: bool,
    pub costing_enabled: bool,
    pub node_move_enabled: bool,
    pub auth_enabled: bool,
    pub logger_enabled: bool,
    pub transaction_runtime_enabled: bool,
    pub execution_trace_enabled: bool,

    /* states */
    pub kernel_debug: KernelDebugModule,
    pub costing: CostingModule,
    pub node_move: NodeMoveModule,
    pub auth: AuthModule,
    pub logger: LoggerModule,
    pub transaction_runtime: TransactionRuntimeModule,
    pub execution_trace: ExecutionTraceModule,
}

impl KernelModuleMixer {
    pub fn standard(
        debug: bool,
        tx_hash: Hash,
        auth_zone_params: AuthZoneParams,
        fee_reserve: SystemLoanFeeReserve,
        fee_table: FeeTable,
        max_call_depth: usize,
        max_kernel_call_depth_traced: Option<usize>,
    ) -> Self {
        Self {
            kernel_debug_enabled: debug,
            costing_enabled: true,
            node_move_enabled: true,
            auth_enabled: true,
            logger_enabled: true,
            transaction_runtime_enabled: true,
            execution_trace_enabled: max_kernel_call_depth_traced.is_some(),
            kernel_debug: KernelDebugModule {},
            costing: CostingModule {
                fee_reserve,
                fee_table,
                max_call_depth,
            },
            node_move: NodeMoveModule {},
            auth: AuthModule {
                params: auth_zone_params.clone(),
            },
            logger: LoggerModule {},
            transaction_runtime: TransactionRuntimeModule { tx_hash },
            execution_trace: ExecutionTraceModule::new(max_kernel_call_depth_traced.unwrap_or(0)),
        }
    }
}

//====================================================================
// NOTE: Modules are applied in the reverse order of initialization!
//====================================================================

impl KernelModule for KernelModuleMixer {
    fn on_init<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        // Enable execution trace
        if api.get_module_state().execution_trace_enabled {
            ExecutionTraceModule::on_init(api)?;
        }

        // Enable transaction runtime
        if api.get_module_state().transaction_runtime_enabled {
            TransactionRuntimeModule::on_init(api)?;
        }

        // Enable logger
        if api.get_module_state().logger_enabled {
            LoggerModule::on_init(api)?;
        }

        // Enable auth
        if api.get_module_state().auth_enabled {
            AuthModule::on_init(api)?;
        }

        // Enable node move
        if api.get_module_state().node_move_enabled {
            NodeMoveModule::on_init(api)?;
        }

        // Enable costing
        if api.get_module_state().costing_enabled {
            CostingModule::on_init(api)?;
        }

        // Enable debug
        if api.get_module_state().kernel_debug_enabled {
            KernelDebugModule::on_init(api)?;
        }

        Ok(())
    }

    fn on_teardown<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        if api.get_module_state().kernel_debug_enabled {
            KernelDebugModule::on_teardown(api)?;
        }
        if api.get_module_state().costing_enabled {
            CostingModule::on_teardown(api)?;
        }
        if api.get_module_state().node_move_enabled {
            NodeMoveModule::on_teardown(api)?;
        }
        if api.get_module_state().auth_enabled {
            AuthModule::on_teardown(api)?;
        }
        if api.get_module_state().logger_enabled {
            LoggerModule::on_teardown(api)?;
        }
        if api.get_module_state().transaction_runtime_enabled {
            TransactionRuntimeModule::on_teardown(api)?;
        }
        if api.get_module_state().execution_trace_enabled {
            ExecutionTraceModule::on_teardown(api)?;
        }
        Ok(())
    }

    fn before_invoke<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        fn_identifier: &FnIdentifier,
        input_size: usize,
    ) -> Result<(), RuntimeError> {
        if api.get_module_state().kernel_debug_enabled {
            KernelDebugModule::before_invoke(api, fn_identifier, input_size)?;
        }
        if api.get_module_state().costing_enabled {
            CostingModule::before_invoke(api, fn_identifier, input_size)?;
        }
        if api.get_module_state().node_move_enabled {
            NodeMoveModule::before_invoke(api, fn_identifier, input_size)?;
        }
        if api.get_module_state().auth_enabled {
            AuthModule::before_invoke(api, fn_identifier, input_size)?;
        }
        if api.get_module_state().logger_enabled {
            LoggerModule::before_invoke(api, fn_identifier, input_size)?;
        }
        if api.get_module_state().transaction_runtime_enabled {
            TransactionRuntimeModule::before_invoke(api, fn_identifier, input_size)?;
        }
        if api.get_module_state().execution_trace_enabled {
            ExecutionTraceModule::before_invoke(api, fn_identifier, input_size)?;
        }
        Ok(())
    }

    fn before_push_frame<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        actor: &ResolvedActor,
        update: &mut CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        if api.get_module_state().kernel_debug_enabled {
            KernelDebugModule::before_push_frame(api, actor, update)?;
        }
        if api.get_module_state().costing_enabled {
            CostingModule::before_push_frame(api, actor, update)?;
        }
        if api.get_module_state().node_move_enabled {
            NodeMoveModule::before_push_frame(api, actor, update)?;
        }
        if api.get_module_state().auth_enabled {
            AuthModule::before_push_frame(api, actor, update)?;
        }
        if api.get_module_state().logger_enabled {
            LoggerModule::before_push_frame(api, actor, update)?;
        }
        if api.get_module_state().transaction_runtime_enabled {
            TransactionRuntimeModule::before_push_frame(api, actor, update)?;
        }
        if api.get_module_state().execution_trace_enabled {
            ExecutionTraceModule::before_push_frame(api, actor, update)?;
        }
        Ok(())
    }

    fn on_execution_start<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        caller: &ResolvedActor,
    ) -> Result<(), RuntimeError> {
        if api.get_module_state().kernel_debug_enabled {
            KernelDebugModule::on_execution_start(api, caller)?;
        }
        if api.get_module_state().costing_enabled {
            CostingModule::on_execution_start(api, caller)?;
        }
        if api.get_module_state().node_move_enabled {
            NodeMoveModule::on_execution_start(api, caller)?;
        }
        if api.get_module_state().auth_enabled {
            AuthModule::on_execution_start(api, caller)?;
        }
        if api.get_module_state().logger_enabled {
            LoggerModule::on_execution_start(api, caller)?;
        }
        if api.get_module_state().transaction_runtime_enabled {
            TransactionRuntimeModule::on_execution_start(api, caller)?;
        }
        if api.get_module_state().execution_trace_enabled {
            ExecutionTraceModule::on_execution_start(api, caller)?;
        }
        Ok(())
    }

    fn on_execution_finish<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        caller: &ResolvedActor,
        update: &CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        if api.get_module_state().kernel_debug_enabled {
            KernelDebugModule::on_execution_finish(api, caller, update)?;
        }
        if api.get_module_state().costing_enabled {
            CostingModule::on_execution_finish(api, caller, update)?;
        }
        if api.get_module_state().node_move_enabled {
            NodeMoveModule::on_execution_finish(api, caller, update)?;
        }
        if api.get_module_state().auth_enabled {
            AuthModule::on_execution_finish(api, caller, update)?;
        }
        if api.get_module_state().logger_enabled {
            LoggerModule::on_execution_finish(api, caller, update)?;
        }
        if api.get_module_state().transaction_runtime_enabled {
            TransactionRuntimeModule::on_execution_finish(api, caller, update)?;
        }
        if api.get_module_state().execution_trace_enabled {
            ExecutionTraceModule::on_execution_finish(api, caller, update)?;
        }
        Ok(())
    }

    fn after_pop_frame<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        if api.get_module_state().kernel_debug_enabled {
            KernelDebugModule::after_pop_frame(api)?;
        }
        if api.get_module_state().costing_enabled {
            CostingModule::after_pop_frame(api)?;
        }
        if api.get_module_state().node_move_enabled {
            NodeMoveModule::after_pop_frame(api)?;
        }
        if api.get_module_state().auth_enabled {
            AuthModule::after_pop_frame(api)?;
        }
        if api.get_module_state().logger_enabled {
            LoggerModule::after_pop_frame(api)?;
        }
        if api.get_module_state().transaction_runtime_enabled {
            TransactionRuntimeModule::after_pop_frame(api)?;
        }
        if api.get_module_state().execution_trace_enabled {
            ExecutionTraceModule::after_pop_frame(api)?;
        }
        Ok(())
    }

    fn after_invoke<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        output_size: usize,
    ) -> Result<(), RuntimeError> {
        if api.get_module_state().kernel_debug_enabled {
            KernelDebugModule::after_invoke(api, output_size)?;
        }
        if api.get_module_state().costing_enabled {
            CostingModule::after_invoke(api, output_size)?;
        }
        if api.get_module_state().node_move_enabled {
            NodeMoveModule::after_invoke(api, output_size)?;
        }
        if api.get_module_state().auth_enabled {
            AuthModule::after_invoke(api, output_size)?;
        }
        if api.get_module_state().logger_enabled {
            LoggerModule::after_invoke(api, output_size)?;
        }
        if api.get_module_state().transaction_runtime_enabled {
            TransactionRuntimeModule::after_invoke(api, output_size)?;
        }
        if api.get_module_state().execution_trace_enabled {
            ExecutionTraceModule::after_invoke(api, output_size)?;
        }
        Ok(())
    }

    fn on_allocate_node_id<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        node_type: &RENodeType,
    ) -> Result<(), RuntimeError> {
        if api.get_module_state().kernel_debug_enabled {
            KernelDebugModule::on_allocate_node_id(api, node_type)?;
        }
        if api.get_module_state().costing_enabled {
            CostingModule::on_allocate_node_id(api, node_type)?;
        }
        if api.get_module_state().node_move_enabled {
            NodeMoveModule::on_allocate_node_id(api, node_type)?;
        }
        if api.get_module_state().auth_enabled {
            AuthModule::on_allocate_node_id(api, node_type)?;
        }
        if api.get_module_state().logger_enabled {
            LoggerModule::on_allocate_node_id(api, node_type)?;
        }
        if api.get_module_state().transaction_runtime_enabled {
            TransactionRuntimeModule::on_allocate_node_id(api, node_type)?;
        }
        if api.get_module_state().execution_trace_enabled {
            ExecutionTraceModule::on_allocate_node_id(api, node_type)?;
        }
        Ok(())
    }

    fn before_create_node<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        node_id: &RENodeId,
        node_init: &RENodeInit,
        node_module_init: &BTreeMap<NodeModuleId, RENodeModuleInit>,
    ) -> Result<(), RuntimeError> {
        if api.get_module_state().kernel_debug_enabled {
            KernelDebugModule::before_create_node(api, node_id, node_init, node_module_init)?;
        }
        if api.get_module_state().costing_enabled {
            CostingModule::before_create_node(api, node_id, node_init, node_module_init)?;
        }
        if api.get_module_state().node_move_enabled {
            NodeMoveModule::before_create_node(api, node_id, node_init, node_module_init)?;
        }
        if api.get_module_state().auth_enabled {
            AuthModule::before_create_node(api, node_id, node_init, node_module_init)?;
        }
        if api.get_module_state().logger_enabled {
            LoggerModule::before_create_node(api, node_id, node_init, node_module_init)?;
        }
        if api.get_module_state().transaction_runtime_enabled {
            TransactionRuntimeModule::before_create_node(
                api,
                node_id,
                node_init,
                node_module_init,
            )?;
        }
        if api.get_module_state().execution_trace_enabled {
            ExecutionTraceModule::before_create_node(api, node_id, node_init, node_module_init)?;
        }
        Ok(())
    }

    fn after_create_node<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        node_id: &RENodeId,
    ) -> Result<(), RuntimeError> {
        if api.get_module_state().kernel_debug_enabled {
            KernelDebugModule::after_create_node(api, node_id)?;
        }
        if api.get_module_state().costing_enabled {
            CostingModule::after_create_node(api, node_id)?;
        }
        if api.get_module_state().node_move_enabled {
            NodeMoveModule::after_create_node(api, node_id)?;
        }
        if api.get_module_state().auth_enabled {
            AuthModule::after_create_node(api, node_id)?;
        }
        if api.get_module_state().logger_enabled {
            LoggerModule::after_create_node(api, node_id)?;
        }
        if api.get_module_state().transaction_runtime_enabled {
            TransactionRuntimeModule::after_create_node(api, node_id)?;
        }
        if api.get_module_state().execution_trace_enabled {
            ExecutionTraceModule::after_create_node(api, node_id)?;
        }
        Ok(())
    }

    fn before_drop_node<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        node_id: &RENodeId,
    ) -> Result<(), RuntimeError> {
        if api.get_module_state().kernel_debug_enabled {
            KernelDebugModule::before_drop_node(api, node_id)?;
        }
        if api.get_module_state().costing_enabled {
            CostingModule::before_drop_node(api, node_id)?;
        }
        if api.get_module_state().node_move_enabled {
            NodeMoveModule::before_drop_node(api, node_id)?;
        }
        if api.get_module_state().auth_enabled {
            AuthModule::before_drop_node(api, node_id)?;
        }
        if api.get_module_state().logger_enabled {
            LoggerModule::before_drop_node(api, node_id)?;
        }
        if api.get_module_state().transaction_runtime_enabled {
            TransactionRuntimeModule::before_drop_node(api, node_id)?;
        }
        if api.get_module_state().execution_trace_enabled {
            ExecutionTraceModule::before_drop_node(api, node_id)?;
        }
        Ok(())
    }

    fn after_drop_node<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        if api.get_module_state().kernel_debug_enabled {
            KernelDebugModule::after_drop_node(api)?;
        }
        if api.get_module_state().costing_enabled {
            CostingModule::after_drop_node(api)?;
        }
        if api.get_module_state().node_move_enabled {
            NodeMoveModule::after_drop_node(api)?;
        }
        if api.get_module_state().auth_enabled {
            AuthModule::after_drop_node(api)?;
        }
        if api.get_module_state().logger_enabled {
            LoggerModule::after_drop_node(api)?;
        }
        if api.get_module_state().transaction_runtime_enabled {
            TransactionRuntimeModule::after_drop_node(api)?;
        }
        if api.get_module_state().execution_trace_enabled {
            ExecutionTraceModule::after_drop_node(api)?;
        }
        Ok(())
    }

    fn before_lock_substate<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        node_id: &RENodeId,
        module_id: &NodeModuleId,
        offset: &SubstateOffset,
        flags: &LockFlags,
    ) -> Result<(), RuntimeError> {
        if api.get_module_state().kernel_debug_enabled {
            KernelDebugModule::before_lock_substate(api, node_id, module_id, offset, flags)?;
        }
        if api.get_module_state().costing_enabled {
            CostingModule::before_lock_substate(api, node_id, module_id, offset, flags)?;
        }
        if api.get_module_state().node_move_enabled {
            NodeMoveModule::before_lock_substate(api, node_id, module_id, offset, flags)?;
        }
        if api.get_module_state().auth_enabled {
            AuthModule::before_lock_substate(api, node_id, module_id, offset, flags)?;
        }
        if api.get_module_state().logger_enabled {
            LoggerModule::before_lock_substate(api, node_id, module_id, offset, flags)?;
        }
        if api.get_module_state().transaction_runtime_enabled {
            TransactionRuntimeModule::before_lock_substate(api, node_id, module_id, offset, flags)?;
        }
        if api.get_module_state().execution_trace_enabled {
            ExecutionTraceModule::before_lock_substate(api, node_id, module_id, offset, flags)?;
        }
        Ok(())
    }

    fn after_lock_substate<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        handle: LockHandle,
        size: usize,
    ) -> Result<(), RuntimeError> {
        if api.get_module_state().kernel_debug_enabled {
            KernelDebugModule::after_lock_substate(api, handle, size)?;
        }
        if api.get_module_state().costing_enabled {
            CostingModule::after_lock_substate(api, handle, size)?;
        }
        if api.get_module_state().node_move_enabled {
            NodeMoveModule::after_lock_substate(api, handle, size)?;
        }
        if api.get_module_state().auth_enabled {
            AuthModule::after_lock_substate(api, handle, size)?;
        }
        if api.get_module_state().logger_enabled {
            LoggerModule::after_lock_substate(api, handle, size)?;
        }
        if api.get_module_state().transaction_runtime_enabled {
            TransactionRuntimeModule::after_lock_substate(api, handle, size)?;
        }
        if api.get_module_state().execution_trace_enabled {
            ExecutionTraceModule::after_lock_substate(api, handle, size)?;
        }
        Ok(())
    }

    fn on_read_substate<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        lock_handle: LockHandle,
        size: usize,
    ) -> Result<(), RuntimeError> {
        if api.get_module_state().kernel_debug_enabled {
            KernelDebugModule::on_read_substate(api, lock_handle, size)?;
        }
        if api.get_module_state().costing_enabled {
            CostingModule::on_read_substate(api, lock_handle, size)?;
        }
        if api.get_module_state().node_move_enabled {
            NodeMoveModule::on_read_substate(api, lock_handle, size)?;
        }
        if api.get_module_state().auth_enabled {
            AuthModule::on_read_substate(api, lock_handle, size)?;
        }
        if api.get_module_state().logger_enabled {
            LoggerModule::on_read_substate(api, lock_handle, size)?;
        }
        if api.get_module_state().transaction_runtime_enabled {
            TransactionRuntimeModule::on_read_substate(api, lock_handle, size)?;
        }
        if api.get_module_state().execution_trace_enabled {
            ExecutionTraceModule::on_read_substate(api, lock_handle, size)?;
        }
        Ok(())
    }

    fn on_write_substate<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        lock_handle: LockHandle,
        size: usize,
    ) -> Result<(), RuntimeError> {
        if api.get_module_state().kernel_debug_enabled {
            KernelDebugModule::on_write_substate(api, lock_handle, size)?;
        }
        if api.get_module_state().costing_enabled {
            CostingModule::on_write_substate(api, lock_handle, size)?;
        }
        if api.get_module_state().node_move_enabled {
            NodeMoveModule::on_write_substate(api, lock_handle, size)?;
        }
        if api.get_module_state().auth_enabled {
            AuthModule::on_write_substate(api, lock_handle, size)?;
        }
        if api.get_module_state().logger_enabled {
            LoggerModule::on_write_substate(api, lock_handle, size)?;
        }
        if api.get_module_state().transaction_runtime_enabled {
            TransactionRuntimeModule::on_write_substate(api, lock_handle, size)?;
        }
        if api.get_module_state().execution_trace_enabled {
            ExecutionTraceModule::on_write_substate(api, lock_handle, size)?;
        }
        Ok(())
    }

    fn on_drop_lock<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        lock_handle: LockHandle,
    ) -> Result<(), RuntimeError> {
        if api.get_module_state().kernel_debug_enabled {
            KernelDebugModule::on_drop_lock(api, lock_handle)?;
        }
        if api.get_module_state().costing_enabled {
            CostingModule::on_drop_lock(api, lock_handle)?;
        }
        if api.get_module_state().node_move_enabled {
            NodeMoveModule::on_drop_lock(api, lock_handle)?;
        }
        if api.get_module_state().auth_enabled {
            AuthModule::on_drop_lock(api, lock_handle)?;
        }
        if api.get_module_state().logger_enabled {
            LoggerModule::on_drop_lock(api, lock_handle)?;
        }
        if api.get_module_state().transaction_runtime_enabled {
            TransactionRuntimeModule::on_drop_lock(api, lock_handle)?;
        }
        if api.get_module_state().execution_trace_enabled {
            ExecutionTraceModule::on_drop_lock(api, lock_handle)?;
        }
        Ok(())
    }

    fn on_wasm_instantiation<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        code: &[u8],
    ) -> Result<(), RuntimeError> {
        if api.get_module_state().kernel_debug_enabled {
            KernelDebugModule::on_wasm_instantiation(api, code)?;
        }
        if api.get_module_state().costing_enabled {
            CostingModule::on_wasm_instantiation(api, code)?;
        }
        if api.get_module_state().node_move_enabled {
            NodeMoveModule::on_wasm_instantiation(api, code)?;
        }
        if api.get_module_state().auth_enabled {
            AuthModule::on_wasm_instantiation(api, code)?;
        }
        if api.get_module_state().logger_enabled {
            LoggerModule::on_wasm_instantiation(api, code)?;
        }
        if api.get_module_state().transaction_runtime_enabled {
            TransactionRuntimeModule::on_wasm_instantiation(api, code)?;
        }
        if api.get_module_state().execution_trace_enabled {
            ExecutionTraceModule::on_wasm_instantiation(api, code)?;
        }
        Ok(())
    }

    fn on_consume_cost_units<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        units: u32,
    ) -> Result<(), RuntimeError> {
        if api.get_module_state().kernel_debug_enabled {
            KernelDebugModule::on_consume_cost_units(api, units)?;
        }
        if api.get_module_state().costing_enabled {
            CostingModule::on_consume_cost_units(api, units)?;
        }
        if api.get_module_state().node_move_enabled {
            NodeMoveModule::on_consume_cost_units(api, units)?;
        }
        if api.get_module_state().auth_enabled {
            AuthModule::on_consume_cost_units(api, units)?;
        }
        if api.get_module_state().logger_enabled {
            LoggerModule::on_consume_cost_units(api, units)?;
        }
        if api.get_module_state().transaction_runtime_enabled {
            TransactionRuntimeModule::on_consume_cost_units(api, units)?;
        }
        if api.get_module_state().execution_trace_enabled {
            ExecutionTraceModule::on_consume_cost_units(api, units)?;
        }
        Ok(())
    }

    fn on_credit_cost_units<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        vault_id: VaultId,
        mut fee: Resource,
        contingent: bool,
    ) -> Result<Resource, RuntimeError> {
        if api.get_module_state().kernel_debug_enabled {
            fee = KernelDebugModule::on_credit_cost_units(api, vault_id, fee, contingent)?;
        }
        if api.get_module_state().costing_enabled {
            fee = CostingModule::on_credit_cost_units(api, vault_id, fee, contingent)?;
        }
        if api.get_module_state().node_move_enabled {
            fee = NodeMoveModule::on_credit_cost_units(api, vault_id, fee, contingent)?;
        }
        if api.get_module_state().auth_enabled {
            fee = AuthModule::on_credit_cost_units(api, vault_id, fee, contingent)?;
        }
        if api.get_module_state().logger_enabled {
            fee = LoggerModule::on_credit_cost_units(api, vault_id, fee, contingent)?;
        }
        if api.get_module_state().transaction_runtime_enabled {
            fee = TransactionRuntimeModule::on_credit_cost_units(api, vault_id, fee, contingent)?;
        }
        if api.get_module_state().execution_trace_enabled {
            fee = ExecutionTraceModule::on_credit_cost_units(api, vault_id, fee, contingent)?;
        }

        Ok(fee)
    }

    fn on_update_instruction_index<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        new_index: usize,
    ) -> Result<(), RuntimeError> {
        if api.get_module_state().kernel_debug_enabled {
            KernelDebugModule::on_update_instruction_index(api, new_index)?;
        }
        if api.get_module_state().costing_enabled {
            CostingModule::on_update_instruction_index(api, new_index)?;
        }
        if api.get_module_state().node_move_enabled {
            NodeMoveModule::on_update_instruction_index(api, new_index)?;
        }
        if api.get_module_state().auth_enabled {
            AuthModule::on_update_instruction_index(api, new_index)?;
        }
        if api.get_module_state().logger_enabled {
            LoggerModule::on_update_instruction_index(api, new_index)?;
        }
        if api.get_module_state().transaction_runtime_enabled {
            TransactionRuntimeModule::on_update_instruction_index(api, new_index)?;
        }
        if api.get_module_state().execution_trace_enabled {
            ExecutionTraceModule::on_update_instruction_index(api, new_index)?;
        }
        Ok(())
    }
}
