use super::actor::ResolvedActor;
use super::kernel_api::KernelModuleApi;
use crate::errors::*;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::LockFlags;
use crate::kernel::module::KernelModule;
use crate::system::kernel_modules::auth::AuthModule;
use crate::system::kernel_modules::costing::CostingModule;
use crate::system::kernel_modules::costing::FeeTable;
use crate::system::kernel_modules::costing::SystemLoanFeeReserve;
use crate::system::kernel_modules::events::EventsModule;
use crate::system::kernel_modules::execution_trace::ExecutionTraceModule;
use crate::system::kernel_modules::kernel_debug::KernelDebugModule;
use crate::system::kernel_modules::logger::LoggerModule;
use crate::system::kernel_modules::node_move::NodeMoveModule;
use crate::system::kernel_modules::transaction_limits::{
    TransactionLimitsConfig, TransactionLimitsModule,
};
use crate::system::kernel_modules::transaction_runtime::TransactionRuntimeModule;
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use crate::types::api::unsafe_api::ClientCostingReason;
use bitflags::bitflags;
use radix_engine_interface::api::types::InvocationIdentifier;
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

bitflags! {
    pub struct EnabledModules: u32 {
        const KERNEL_DEBUG = 0x1 << 0;
        const COSTING = 0x01 << 1;
        const NODE_MOVE = 0x01 << 2;
        const AUTH = 0x01 << 3;
        const LOGGER = 0x01 << 4;
        const TRANSACTION_RUNTIME = 0x01 << 5;
        const EXECUTION_TRACE = 0x01 << 6;
        const TRANSACTION_LIMITS = 0x01 << 7;
        const EVENTS = 0x01 << 8;
    }
}

pub struct KernelModuleMixer {
    /* flags */
    pub enabled_modules: EnabledModules,

    /* states */
    pub kernel_debug: KernelDebugModule,
    pub costing: CostingModule,
    pub node_move: NodeMoveModule,
    pub auth: AuthModule,
    pub logger: LoggerModule,
    pub transaction_runtime: TransactionRuntimeModule,
    pub execution_trace: ExecutionTraceModule,
    pub transaction_limits: TransactionLimitsModule,
    pub events: EventsModule,
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
        max_wasm_memory: usize,
        max_wasm_memory_per_call_frame: usize,
        max_substate_reads: usize,
        max_substate_writes: usize,
    ) -> Self {
        let mut modules = EnabledModules::empty();
        if debug {
            modules |= EnabledModules::KERNEL_DEBUG
        };
        modules |= EnabledModules::COSTING;
        modules |= EnabledModules::NODE_MOVE;
        modules |= EnabledModules::AUTH;
        modules |= EnabledModules::LOGGER;
        modules |= EnabledModules::TRANSACTION_RUNTIME;
        if max_kernel_call_depth_traced.is_some() {
            modules |= EnabledModules::EXECUTION_TRACE;
        }
        modules |= EnabledModules::TRANSACTION_LIMITS;
        modules |= EnabledModules::EVENTS;

        Self {
            enabled_modules: modules,
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
            logger: LoggerModule::default(),
            transaction_runtime: TransactionRuntimeModule { tx_hash },
            transaction_limits: TransactionLimitsModule::new(TransactionLimitsConfig {
                max_wasm_memory,
                max_wasm_memory_per_call_frame,
                max_substate_reads,
                max_substate_writes,
            }),
            execution_trace: ExecutionTraceModule::new(max_kernel_call_depth_traced.unwrap_or(0)),
            events: EventsModule::default(),
        }
    }
}

//====================================================================
// NOTE: Modules are applied in the reverse order of initialization!
//====================================================================

impl KernelModule for KernelModuleMixer {
    fn on_init<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        let modules: EnabledModules = api.kernel_get_module_state().enabled_modules;

        // Enable transaction limits
        if modules.contains(EnabledModules::TRANSACTION_LIMITS) {
            TransactionLimitsModule::on_init(api)?;
        }

        // Enable execution trace
        if modules.contains(EnabledModules::KERNEL_DEBUG) {
            ExecutionTraceModule::on_init(api)?;
        }

        // Enable transaction runtime
        if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
            TransactionRuntimeModule::on_init(api)?;
        }

        // Enable logger
        if modules.contains(EnabledModules::LOGGER) {
            LoggerModule::on_init(api)?;
        }

        // Enable auth
        if modules.contains(EnabledModules::AUTH) {
            AuthModule::on_init(api)?;
        }

        // Enable node move
        if modules.contains(EnabledModules::NODE_MOVE) {
            NodeMoveModule::on_init(api)?;
        }

        // Enable costing
        if modules.contains(EnabledModules::COSTING) {
            CostingModule::on_init(api)?;
        }

        // Enable debug
        if modules.contains(EnabledModules::KERNEL_DEBUG) {
            KernelDebugModule::on_init(api)?;
        }

        // Enable events
        if modules.contains(EnabledModules::EVENTS) {
            EventsModule::on_init(api)?;
        }

        Ok(())
    }

    fn on_teardown<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        let modules: EnabledModules = api.kernel_get_module_state().enabled_modules;
        if modules.contains(EnabledModules::KERNEL_DEBUG) {
            KernelDebugModule::on_teardown(api)?;
        }
        if modules.contains(EnabledModules::COSTING) {
            CostingModule::on_teardown(api)?;
        }
        if modules.contains(EnabledModules::NODE_MOVE) {
            NodeMoveModule::on_teardown(api)?;
        }
        if modules.contains(EnabledModules::AUTH) {
            AuthModule::on_teardown(api)?;
        }
        if modules.contains(EnabledModules::LOGGER) {
            LoggerModule::on_teardown(api)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
            TransactionRuntimeModule::on_teardown(api)?;
        }
        if modules.contains(EnabledModules::EXECUTION_TRACE) {
            ExecutionTraceModule::on_teardown(api)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_LIMITS) {
            TransactionLimitsModule::on_teardown(api)?;
        }
        if modules.contains(EnabledModules::EVENTS) {
            EventsModule::on_teardown(api)?;
        }
        Ok(())
    }

    fn before_invoke<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        identifier: &InvocationIdentifier,
        input_size: usize,
    ) -> Result<(), RuntimeError> {
        let modules: EnabledModules = api.kernel_get_module_state().enabled_modules;
        if modules.contains(EnabledModules::KERNEL_DEBUG) {
            KernelDebugModule::before_invoke(api, identifier, input_size)?;
        }
        if modules.contains(EnabledModules::COSTING) {
            CostingModule::before_invoke(api, identifier, input_size)?;
        }
        if modules.contains(EnabledModules::NODE_MOVE) {
            NodeMoveModule::before_invoke(api, identifier, input_size)?;
        }
        if modules.contains(EnabledModules::AUTH) {
            AuthModule::before_invoke(api, identifier, input_size)?;
        }
        if modules.contains(EnabledModules::LOGGER) {
            LoggerModule::before_invoke(api, identifier, input_size)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
            TransactionRuntimeModule::before_invoke(api, identifier, input_size)?;
        }
        if modules.contains(EnabledModules::EXECUTION_TRACE) {
            ExecutionTraceModule::before_invoke(api, identifier, input_size)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_LIMITS) {
            TransactionLimitsModule::before_invoke(api, identifier, input_size)?;
        }
        if modules.contains(EnabledModules::EVENTS) {
            EventsModule::before_invoke(api, identifier, input_size)?;
        }
        Ok(())
    }

    fn before_push_frame<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        actor: &Option<ResolvedActor>,
        update: &mut CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        let modules: EnabledModules = api.kernel_get_module_state().enabled_modules;
        if modules.contains(EnabledModules::KERNEL_DEBUG) {
            KernelDebugModule::before_push_frame(api, actor, update)?;
        }
        if modules.contains(EnabledModules::COSTING) {
            CostingModule::before_push_frame(api, actor, update)?;
        }
        if modules.contains(EnabledModules::NODE_MOVE) {
            NodeMoveModule::before_push_frame(api, actor, update)?;
        }
        if modules.contains(EnabledModules::AUTH) {
            AuthModule::before_push_frame(api, actor, update)?;
        }
        if modules.contains(EnabledModules::LOGGER) {
            LoggerModule::before_push_frame(api, actor, update)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
            TransactionRuntimeModule::before_push_frame(api, actor, update)?;
        }
        if modules.contains(EnabledModules::EXECUTION_TRACE) {
            ExecutionTraceModule::before_push_frame(api, actor, update)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_LIMITS) {
            TransactionLimitsModule::before_push_frame(api, actor, update)?;
        }
        if modules.contains(EnabledModules::EVENTS) {
            EventsModule::before_push_frame(api, actor, update)?;
        }
        Ok(())
    }

    fn on_execution_start<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        caller: &Option<ResolvedActor>,
    ) -> Result<(), RuntimeError> {
        let modules: EnabledModules = api.kernel_get_module_state().enabled_modules;
        if modules.contains(EnabledModules::KERNEL_DEBUG) {
            KernelDebugModule::on_execution_start(api, caller)?;
        }
        if modules.contains(EnabledModules::COSTING) {
            CostingModule::on_execution_start(api, caller)?;
        }
        if modules.contains(EnabledModules::NODE_MOVE) {
            NodeMoveModule::on_execution_start(api, caller)?;
        }
        if modules.contains(EnabledModules::AUTH) {
            AuthModule::on_execution_start(api, caller)?;
        }
        if modules.contains(EnabledModules::LOGGER) {
            LoggerModule::on_execution_start(api, caller)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
            TransactionRuntimeModule::on_execution_start(api, caller)?;
        }
        if modules.contains(EnabledModules::EXECUTION_TRACE) {
            ExecutionTraceModule::on_execution_start(api, caller)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_LIMITS) {
            TransactionLimitsModule::on_execution_start(api, caller)?;
        }
        if modules.contains(EnabledModules::EVENTS) {
            EventsModule::on_execution_start(api, caller)?;
        }
        Ok(())
    }

    fn on_execution_finish<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        caller: &Option<ResolvedActor>,
        update: &CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        let modules: EnabledModules = api.kernel_get_module_state().enabled_modules;
        if modules.contains(EnabledModules::KERNEL_DEBUG) {
            KernelDebugModule::on_execution_finish(api, caller, update)?;
        }
        if modules.contains(EnabledModules::COSTING) {
            CostingModule::on_execution_finish(api, caller, update)?;
        }
        if modules.contains(EnabledModules::NODE_MOVE) {
            NodeMoveModule::on_execution_finish(api, caller, update)?;
        }
        if modules.contains(EnabledModules::AUTH) {
            AuthModule::on_execution_finish(api, caller, update)?;
        }
        if modules.contains(EnabledModules::LOGGER) {
            LoggerModule::on_execution_finish(api, caller, update)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
            TransactionRuntimeModule::on_execution_finish(api, caller, update)?;
        }
        if modules.contains(EnabledModules::EXECUTION_TRACE) {
            ExecutionTraceModule::on_execution_finish(api, caller, update)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_LIMITS) {
            TransactionLimitsModule::on_execution_finish(api, caller, update)?;
        }
        if modules.contains(EnabledModules::EVENTS) {
            EventsModule::on_execution_finish(api, caller, update)?;
        }
        Ok(())
    }

    fn after_pop_frame<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        let modules: EnabledModules = api.kernel_get_module_state().enabled_modules;
        if modules.contains(EnabledModules::KERNEL_DEBUG) {
            KernelDebugModule::after_pop_frame(api)?;
        }
        if modules.contains(EnabledModules::COSTING) {
            CostingModule::after_pop_frame(api)?;
        }
        if modules.contains(EnabledModules::NODE_MOVE) {
            NodeMoveModule::after_pop_frame(api)?;
        }
        if modules.contains(EnabledModules::AUTH) {
            AuthModule::after_pop_frame(api)?;
        }
        if modules.contains(EnabledModules::LOGGER) {
            LoggerModule::after_pop_frame(api)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
            TransactionRuntimeModule::after_pop_frame(api)?;
        }
        if modules.contains(EnabledModules::EXECUTION_TRACE) {
            ExecutionTraceModule::after_pop_frame(api)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_LIMITS) {
            TransactionLimitsModule::after_pop_frame(api)?;
        }
        if modules.contains(EnabledModules::EVENTS) {
            EventsModule::after_pop_frame(api)?;
        }
        Ok(())
    }

    fn after_invoke<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        output_size: usize,
    ) -> Result<(), RuntimeError> {
        let modules: EnabledModules = api.kernel_get_module_state().enabled_modules;
        if modules.contains(EnabledModules::KERNEL_DEBUG) {
            KernelDebugModule::after_invoke(api, output_size)?;
        }
        if modules.contains(EnabledModules::COSTING) {
            CostingModule::after_invoke(api, output_size)?;
        }
        if modules.contains(EnabledModules::NODE_MOVE) {
            NodeMoveModule::after_invoke(api, output_size)?;
        }
        if modules.contains(EnabledModules::AUTH) {
            AuthModule::after_invoke(api, output_size)?;
        }
        if modules.contains(EnabledModules::LOGGER) {
            LoggerModule::after_invoke(api, output_size)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
            TransactionRuntimeModule::after_invoke(api, output_size)?;
        }
        if modules.contains(EnabledModules::EXECUTION_TRACE) {
            ExecutionTraceModule::after_invoke(api, output_size)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_LIMITS) {
            TransactionLimitsModule::after_invoke(api, output_size)?;
        }
        if modules.contains(EnabledModules::EVENTS) {
            EventsModule::after_invoke(api, output_size)?;
        }
        Ok(())
    }

    fn on_allocate_node_id<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        node_type: &RENodeType,
    ) -> Result<(), RuntimeError> {
        let modules: EnabledModules = api.kernel_get_module_state().enabled_modules;
        if modules.contains(EnabledModules::KERNEL_DEBUG) {
            KernelDebugModule::on_allocate_node_id(api, node_type)?;
        }
        if modules.contains(EnabledModules::COSTING) {
            CostingModule::on_allocate_node_id(api, node_type)?;
        }
        if modules.contains(EnabledModules::NODE_MOVE) {
            NodeMoveModule::on_allocate_node_id(api, node_type)?;
        }
        if modules.contains(EnabledModules::AUTH) {
            AuthModule::on_allocate_node_id(api, node_type)?;
        }
        if modules.contains(EnabledModules::LOGGER) {
            LoggerModule::on_allocate_node_id(api, node_type)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
            TransactionRuntimeModule::on_allocate_node_id(api, node_type)?;
        }
        if modules.contains(EnabledModules::EXECUTION_TRACE) {
            ExecutionTraceModule::on_allocate_node_id(api, node_type)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_LIMITS) {
            TransactionLimitsModule::on_allocate_node_id(api, node_type)?;
        }
        if modules.contains(EnabledModules::EVENTS) {
            EventsModule::on_allocate_node_id(api, node_type)?;
        }
        Ok(())
    }

    fn before_create_node<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        node_id: &RENodeId,
        node_init: &RENodeInit,
        node_module_init: &BTreeMap<NodeModuleId, RENodeModuleInit>,
    ) -> Result<(), RuntimeError> {
        let modules: EnabledModules = api.kernel_get_module_state().enabled_modules;
        if modules.contains(EnabledModules::KERNEL_DEBUG) {
            KernelDebugModule::before_create_node(api, node_id, node_init, node_module_init)?;
        }
        if modules.contains(EnabledModules::COSTING) {
            CostingModule::before_create_node(api, node_id, node_init, node_module_init)?;
        }
        if modules.contains(EnabledModules::NODE_MOVE) {
            NodeMoveModule::before_create_node(api, node_id, node_init, node_module_init)?;
        }
        if modules.contains(EnabledModules::AUTH) {
            AuthModule::before_create_node(api, node_id, node_init, node_module_init)?;
        }
        if modules.contains(EnabledModules::LOGGER) {
            LoggerModule::before_create_node(api, node_id, node_init, node_module_init)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
            TransactionRuntimeModule::before_create_node(
                api,
                node_id,
                node_init,
                node_module_init,
            )?;
        }
        if modules.contains(EnabledModules::EXECUTION_TRACE) {
            ExecutionTraceModule::before_create_node(api, node_id, node_init, node_module_init)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_LIMITS) {
            TransactionLimitsModule::before_create_node(api, node_id, node_init, node_module_init)?;
        }
        if modules.contains(EnabledModules::EVENTS) {
            EventsModule::before_create_node(api, node_id, node_init, node_module_init)?;
        }
        Ok(())
    }

    fn after_create_node<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        node_id: &RENodeId,
    ) -> Result<(), RuntimeError> {
        let modules: EnabledModules = api.kernel_get_module_state().enabled_modules;
        if modules.contains(EnabledModules::KERNEL_DEBUG) {
            KernelDebugModule::after_create_node(api, node_id)?;
        }
        if modules.contains(EnabledModules::COSTING) {
            CostingModule::after_create_node(api, node_id)?;
        }
        if modules.contains(EnabledModules::NODE_MOVE) {
            NodeMoveModule::after_create_node(api, node_id)?;
        }
        if modules.contains(EnabledModules::AUTH) {
            AuthModule::after_create_node(api, node_id)?;
        }
        if modules.contains(EnabledModules::LOGGER) {
            LoggerModule::after_create_node(api, node_id)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
            TransactionRuntimeModule::after_create_node(api, node_id)?;
        }
        if modules.contains(EnabledModules::EXECUTION_TRACE) {
            ExecutionTraceModule::after_create_node(api, node_id)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_LIMITS) {
            TransactionLimitsModule::after_create_node(api, node_id)?;
        }
        if modules.contains(EnabledModules::EVENTS) {
            EventsModule::after_create_node(api, node_id)?;
        }
        Ok(())
    }

    fn before_drop_node<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        node_id: &RENodeId,
    ) -> Result<(), RuntimeError> {
        let modules: EnabledModules = api.kernel_get_module_state().enabled_modules;
        if modules.contains(EnabledModules::KERNEL_DEBUG) {
            KernelDebugModule::before_drop_node(api, node_id)?;
        }
        if modules.contains(EnabledModules::COSTING) {
            CostingModule::before_drop_node(api, node_id)?;
        }
        if modules.contains(EnabledModules::NODE_MOVE) {
            NodeMoveModule::before_drop_node(api, node_id)?;
        }
        if modules.contains(EnabledModules::AUTH) {
            AuthModule::before_drop_node(api, node_id)?;
        }
        if modules.contains(EnabledModules::LOGGER) {
            LoggerModule::before_drop_node(api, node_id)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
            TransactionRuntimeModule::before_drop_node(api, node_id)?;
        }
        if modules.contains(EnabledModules::EXECUTION_TRACE) {
            ExecutionTraceModule::before_drop_node(api, node_id)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_LIMITS) {
            TransactionLimitsModule::before_drop_node(api, node_id)?;
        }
        if modules.contains(EnabledModules::EVENTS) {
            EventsModule::before_drop_node(api, node_id)?;
        }
        Ok(())
    }

    fn after_drop_node<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        let modules: EnabledModules = api.kernel_get_module_state().enabled_modules;
        if modules.contains(EnabledModules::KERNEL_DEBUG) {
            KernelDebugModule::after_drop_node(api)?;
        }
        if modules.contains(EnabledModules::COSTING) {
            CostingModule::after_drop_node(api)?;
        }
        if modules.contains(EnabledModules::NODE_MOVE) {
            NodeMoveModule::after_drop_node(api)?;
        }
        if modules.contains(EnabledModules::AUTH) {
            AuthModule::after_drop_node(api)?;
        }
        if modules.contains(EnabledModules::LOGGER) {
            LoggerModule::after_drop_node(api)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
            TransactionRuntimeModule::after_drop_node(api)?;
        }
        if modules.contains(EnabledModules::EXECUTION_TRACE) {
            ExecutionTraceModule::after_drop_node(api)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_LIMITS) {
            TransactionLimitsModule::after_drop_node(api)?;
        }
        if modules.contains(EnabledModules::EVENTS) {
            EventsModule::after_drop_node(api)?;
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
        let modules: EnabledModules = api.kernel_get_module_state().enabled_modules;
        if modules.contains(EnabledModules::KERNEL_DEBUG) {
            KernelDebugModule::before_lock_substate(api, node_id, module_id, offset, flags)?;
        }
        if modules.contains(EnabledModules::COSTING) {
            CostingModule::before_lock_substate(api, node_id, module_id, offset, flags)?;
        }
        if modules.contains(EnabledModules::NODE_MOVE) {
            NodeMoveModule::before_lock_substate(api, node_id, module_id, offset, flags)?;
        }
        if modules.contains(EnabledModules::AUTH) {
            AuthModule::before_lock_substate(api, node_id, module_id, offset, flags)?;
        }
        if modules.contains(EnabledModules::LOGGER) {
            LoggerModule::before_lock_substate(api, node_id, module_id, offset, flags)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
            TransactionRuntimeModule::before_lock_substate(api, node_id, module_id, offset, flags)?;
        }
        if modules.contains(EnabledModules::EXECUTION_TRACE) {
            ExecutionTraceModule::before_lock_substate(api, node_id, module_id, offset, flags)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_LIMITS) {
            TransactionLimitsModule::before_lock_substate(api, node_id, module_id, offset, flags)?;
        }
        if modules.contains(EnabledModules::EVENTS) {
            EventsModule::before_lock_substate(api, node_id, module_id, offset, flags)?;
        }
        Ok(())
    }

    fn after_lock_substate<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        handle: LockHandle,
        size: usize,
    ) -> Result<(), RuntimeError> {
        let modules: EnabledModules = api.kernel_get_module_state().enabled_modules;
        if modules.contains(EnabledModules::KERNEL_DEBUG) {
            KernelDebugModule::after_lock_substate(api, handle, size)?;
        }
        if modules.contains(EnabledModules::COSTING) {
            CostingModule::after_lock_substate(api, handle, size)?;
        }
        if modules.contains(EnabledModules::NODE_MOVE) {
            NodeMoveModule::after_lock_substate(api, handle, size)?;
        }
        if modules.contains(EnabledModules::AUTH) {
            AuthModule::after_lock_substate(api, handle, size)?;
        }
        if modules.contains(EnabledModules::LOGGER) {
            LoggerModule::after_lock_substate(api, handle, size)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
            TransactionRuntimeModule::after_lock_substate(api, handle, size)?;
        }
        if modules.contains(EnabledModules::EXECUTION_TRACE) {
            ExecutionTraceModule::after_lock_substate(api, handle, size)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_LIMITS) {
            TransactionLimitsModule::after_lock_substate(api, handle, size)?;
        }
        if modules.contains(EnabledModules::EVENTS) {
            EventsModule::after_lock_substate(api, handle, size)?;
        }
        Ok(())
    }

    fn on_read_substate<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        lock_handle: LockHandle,
        size: usize,
    ) -> Result<(), RuntimeError> {
        let modules: EnabledModules = api.kernel_get_module_state().enabled_modules;
        if modules.contains(EnabledModules::KERNEL_DEBUG) {
            KernelDebugModule::on_read_substate(api, lock_handle, size)?;
        }
        if modules.contains(EnabledModules::COSTING) {
            CostingModule::on_read_substate(api, lock_handle, size)?;
        }
        if modules.contains(EnabledModules::NODE_MOVE) {
            NodeMoveModule::on_read_substate(api, lock_handle, size)?;
        }
        if modules.contains(EnabledModules::AUTH) {
            AuthModule::on_read_substate(api, lock_handle, size)?;
        }
        if modules.contains(EnabledModules::LOGGER) {
            LoggerModule::on_read_substate(api, lock_handle, size)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
            TransactionRuntimeModule::on_read_substate(api, lock_handle, size)?;
        }
        if modules.contains(EnabledModules::EXECUTION_TRACE) {
            ExecutionTraceModule::on_read_substate(api, lock_handle, size)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_LIMITS) {
            TransactionLimitsModule::on_read_substate(api, lock_handle, size)?;
        }
        if modules.contains(EnabledModules::EVENTS) {
            EventsModule::on_read_substate(api, lock_handle, size)?;
        }
        Ok(())
    }

    fn on_write_substate<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        lock_handle: LockHandle,
        size: usize,
    ) -> Result<(), RuntimeError> {
        let modules: EnabledModules = api.kernel_get_module_state().enabled_modules;
        if modules.contains(EnabledModules::KERNEL_DEBUG) {
            KernelDebugModule::on_write_substate(api, lock_handle, size)?;
        }
        if modules.contains(EnabledModules::COSTING) {
            CostingModule::on_write_substate(api, lock_handle, size)?;
        }
        if modules.contains(EnabledModules::NODE_MOVE) {
            NodeMoveModule::on_write_substate(api, lock_handle, size)?;
        }
        if modules.contains(EnabledModules::AUTH) {
            AuthModule::on_write_substate(api, lock_handle, size)?;
        }
        if modules.contains(EnabledModules::LOGGER) {
            LoggerModule::on_write_substate(api, lock_handle, size)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
            TransactionRuntimeModule::on_write_substate(api, lock_handle, size)?;
        }
        if modules.contains(EnabledModules::EXECUTION_TRACE) {
            ExecutionTraceModule::on_write_substate(api, lock_handle, size)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_LIMITS) {
            TransactionLimitsModule::on_write_substate(api, lock_handle, size)?;
        }
        if modules.contains(EnabledModules::EVENTS) {
            EventsModule::on_write_substate(api, lock_handle, size)?;
        }
        Ok(())
    }

    fn on_drop_lock<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        lock_handle: LockHandle,
    ) -> Result<(), RuntimeError> {
        let modules: EnabledModules = api.kernel_get_module_state().enabled_modules;
        if modules.contains(EnabledModules::KERNEL_DEBUG) {
            KernelDebugModule::on_drop_lock(api, lock_handle)?;
        }
        if modules.contains(EnabledModules::COSTING) {
            CostingModule::on_drop_lock(api, lock_handle)?;
        }
        if modules.contains(EnabledModules::NODE_MOVE) {
            NodeMoveModule::on_drop_lock(api, lock_handle)?;
        }
        if modules.contains(EnabledModules::AUTH) {
            AuthModule::on_drop_lock(api, lock_handle)?;
        }
        if modules.contains(EnabledModules::LOGGER) {
            LoggerModule::on_drop_lock(api, lock_handle)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
            TransactionRuntimeModule::on_drop_lock(api, lock_handle)?;
        }
        if modules.contains(EnabledModules::EXECUTION_TRACE) {
            ExecutionTraceModule::on_drop_lock(api, lock_handle)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_LIMITS) {
            TransactionLimitsModule::on_drop_lock(api, lock_handle)?;
        }
        if modules.contains(EnabledModules::EVENTS) {
            EventsModule::on_drop_lock(api, lock_handle)?;
        }
        Ok(())
    }

    fn on_consume_cost_units<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        units: u32,
        reason: ClientCostingReason,
    ) -> Result<(), RuntimeError> {
        let modules: EnabledModules = api.kernel_get_module_state().enabled_modules;
        if modules.contains(EnabledModules::KERNEL_DEBUG) {
            KernelDebugModule::on_consume_cost_units(api, units, reason)?;
        }
        if modules.contains(EnabledModules::COSTING) {
            CostingModule::on_consume_cost_units(api, units, reason)?;
        }
        if modules.contains(EnabledModules::NODE_MOVE) {
            NodeMoveModule::on_consume_cost_units(api, units, reason)?;
        }
        if modules.contains(EnabledModules::AUTH) {
            AuthModule::on_consume_cost_units(api, units, reason)?;
        }
        if modules.contains(EnabledModules::LOGGER) {
            LoggerModule::on_consume_cost_units(api, units, reason)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
            TransactionRuntimeModule::on_consume_cost_units(api, units, reason)?;
        }
        if modules.contains(EnabledModules::EXECUTION_TRACE) {
            ExecutionTraceModule::on_consume_cost_units(api, units, reason)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_LIMITS) {
            TransactionLimitsModule::on_consume_cost_units(api, units, reason)?;
        }
        if modules.contains(EnabledModules::EVENTS) {
            EventsModule::on_consume_cost_units(api, units, reason)?;
        }
        Ok(())
    }

    fn on_credit_cost_units<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        vault_id: VaultId,
        mut fee: Resource,
        contingent: bool,
    ) -> Result<Resource, RuntimeError> {
        let modules: EnabledModules = api.kernel_get_module_state().enabled_modules;
        if modules.contains(EnabledModules::KERNEL_DEBUG) {
            fee = KernelDebugModule::on_credit_cost_units(api, vault_id, fee, contingent)?;
        }
        if modules.contains(EnabledModules::COSTING) {
            fee = CostingModule::on_credit_cost_units(api, vault_id, fee, contingent)?;
        }
        if modules.contains(EnabledModules::NODE_MOVE) {
            fee = NodeMoveModule::on_credit_cost_units(api, vault_id, fee, contingent)?;
        }
        if modules.contains(EnabledModules::AUTH) {
            fee = AuthModule::on_credit_cost_units(api, vault_id, fee, contingent)?;
        }
        if modules.contains(EnabledModules::LOGGER) {
            fee = LoggerModule::on_credit_cost_units(api, vault_id, fee, contingent)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
            fee = TransactionRuntimeModule::on_credit_cost_units(api, vault_id, fee, contingent)?;
        }
        if modules.contains(EnabledModules::EXECUTION_TRACE) {
            fee = ExecutionTraceModule::on_credit_cost_units(api, vault_id, fee, contingent)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_LIMITS) {
            fee = TransactionLimitsModule::on_credit_cost_units(api, vault_id, fee, contingent)?;
        }
        if modules.contains(EnabledModules::EVENTS) {
            fee = EventsModule::on_credit_cost_units(api, vault_id, fee, contingent)?;
        }
        Ok(fee)
    }

    fn on_update_instruction_index<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        new_index: usize,
    ) -> Result<(), RuntimeError> {
        let modules: EnabledModules = api.kernel_get_module_state().enabled_modules;
        if modules.contains(EnabledModules::KERNEL_DEBUG) {
            KernelDebugModule::on_update_instruction_index(api, new_index)?;
        }
        if modules.contains(EnabledModules::COSTING) {
            CostingModule::on_update_instruction_index(api, new_index)?;
        }
        if modules.contains(EnabledModules::NODE_MOVE) {
            NodeMoveModule::on_update_instruction_index(api, new_index)?;
        }
        if modules.contains(EnabledModules::AUTH) {
            AuthModule::on_update_instruction_index(api, new_index)?;
        }
        if modules.contains(EnabledModules::LOGGER) {
            LoggerModule::on_update_instruction_index(api, new_index)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
            TransactionRuntimeModule::on_update_instruction_index(api, new_index)?;
        }
        if modules.contains(EnabledModules::EXECUTION_TRACE) {
            ExecutionTraceModule::on_update_instruction_index(api, new_index)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_LIMITS) {
            TransactionLimitsModule::on_update_instruction_index(api, new_index)?;
        }
        if modules.contains(EnabledModules::EVENTS) {
            EventsModule::on_update_instruction_index(api, new_index)?;
        }
        Ok(())
    }

    fn on_update_wasm_memory_usage<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        consumed_memory: usize,
    ) -> Result<(), RuntimeError> {
        let modules: EnabledModules = api.kernel_get_module_state().enabled_modules;
        if modules.contains(EnabledModules::KERNEL_DEBUG) {
            KernelDebugModule::on_update_wasm_memory_usage(api, consumed_memory)?;
        }
        if modules.contains(EnabledModules::COSTING) {
            CostingModule::on_update_wasm_memory_usage(api, consumed_memory)?;
        }
        if modules.contains(EnabledModules::NODE_MOVE) {
            NodeMoveModule::on_update_wasm_memory_usage(api, consumed_memory)?;
        }
        if modules.contains(EnabledModules::AUTH) {
            AuthModule::on_update_wasm_memory_usage(api, consumed_memory)?;
        }
        if modules.contains(EnabledModules::LOGGER) {
            LoggerModule::on_update_wasm_memory_usage(api, consumed_memory)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
            TransactionRuntimeModule::on_update_wasm_memory_usage(api, consumed_memory)?;
        }
        if modules.contains(EnabledModules::EXECUTION_TRACE) {
            ExecutionTraceModule::on_update_wasm_memory_usage(api, consumed_memory)?;
        }
        if modules.contains(EnabledModules::TRANSACTION_LIMITS) {
            TransactionLimitsModule::on_update_wasm_memory_usage(api, consumed_memory)?;
        }
        if modules.contains(EnabledModules::EVENTS) {
            EventsModule::on_update_wasm_memory_usage(api, consumed_memory)?;
        }
        Ok(())
    }
}
