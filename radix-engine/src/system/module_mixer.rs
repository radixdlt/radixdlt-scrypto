use crate::errors::*;
use crate::kernel::actor::Actor;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::KernelApi;
use crate::kernel::kernel_api::KernelInvocation;
use crate::system::module::SystemModule;
use crate::system::system_callback::SystemConfig;
use crate::system::system_callback_api::SystemCallbackObject;
use crate::system::system_modules::auth::AuthModule;
use crate::system::system_modules::costing::CostingModule;
use crate::system::system_modules::costing::FeeTable;
use crate::system::system_modules::costing::SystemLoanFeeReserve;
use crate::system::system_modules::events::EventsModule;
use crate::system::system_modules::execution_trace::ExecutionTraceModule;
use crate::system::system_modules::kernel_trace::KernelTraceModule;
use crate::system::system_modules::logger::LoggerModule;
use crate::system::system_modules::node_move::NodeMoveModule;
use crate::system::system_modules::transaction_limits::{
    TransactionLimitsConfig, TransactionLimitsModule,
};
use crate::system::system_modules::transaction_runtime::TransactionRuntimeModule;
use crate::system::system_modules::virtualization::VirtualizationModule;
use crate::track::interface::NodeSubstates;
use crate::transaction::ExecutionConfig;
use crate::types::*;
use bitflags::bitflags;
use paste::paste;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::crypto::Hash;
use resources_tracker_macro::trace_resources;
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

pub struct SystemModuleMixer {
    /* flags */
    pub enabled_modules: EnabledModules,

    /* states */
    pub kernel_debug: KernelTraceModule,
    pub costing: CostingModule,
    pub node_move: NodeMoveModule,
    pub auth: AuthModule,
    pub logger: LoggerModule,
    pub transaction_runtime: TransactionRuntimeModule,
    pub execution_trace: ExecutionTraceModule,
    pub transaction_limits: TransactionLimitsModule,
    pub events: EventsModule,
    pub virtualization: VirtualizationModule,
}

// Macro generates default modules dispatches call based on passed function name and arguments.
macro_rules! internal_call_dispatch {
    ($api:ident, $fn:ident ( $($param:ident),*) ) => {
        paste! {
        {
            let modules: EnabledModules = $api.kernel_get_system().modules.enabled_modules;
            if modules.contains(EnabledModules::KERNEL_DEBUG) {
                KernelTraceModule::[< $fn >]($($param, )*)?;
            }
            if modules.contains(EnabledModules::COSTING) {
                CostingModule::[< $fn >]($($param, )*)?;
            }
            if modules.contains(EnabledModules::NODE_MOVE) {
                NodeMoveModule::[< $fn >]($($param, )*)?;
            }
            if modules.contains(EnabledModules::AUTH) {
                AuthModule::[< $fn >]($($param, )*)?;
            }
            if modules.contains(EnabledModules::LOGGER) {
                LoggerModule::[< $fn >]($($param, )*)?;
            }
            if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
                TransactionRuntimeModule::[< $fn >]($($param, )*)?;
            }
            if modules.contains(EnabledModules::EXECUTION_TRACE) {
                ExecutionTraceModule::[< $fn >]($($param, )*)?;
            }
            if modules.contains(EnabledModules::TRANSACTION_LIMITS) {
                TransactionLimitsModule::[< $fn >]($($param, )*)?;
            }
            if modules.contains(EnabledModules::EVENTS) {
                EventsModule::[< $fn >]($($param, )*)?;
            }
            Ok(())
        }
    }};
}

impl SystemModuleMixer {
    pub fn standard(
        tx_hash: Hash,
        auth_zone_params: AuthZoneParams,
        fee_reserve: SystemLoanFeeReserve,
        fee_table: FeeTable,
        payload_len: usize,
        num_of_signatures: usize,
        execution_config: &ExecutionConfig,
    ) -> Self {
        let mut modules = EnabledModules::empty();

        if execution_config.kernel_trace {
            modules |= EnabledModules::KERNEL_DEBUG
        };

        if execution_config.execution_trace.is_some() {
            modules |= EnabledModules::EXECUTION_TRACE;
        }

        if !execution_config.genesis {
            modules |= EnabledModules::COSTING;
            modules |= EnabledModules::AUTH;
            modules |= EnabledModules::TRANSACTION_LIMITS;
        }

        modules |= EnabledModules::NODE_MOVE;
        modules |= EnabledModules::LOGGER;
        modules |= EnabledModules::TRANSACTION_RUNTIME;
        modules |= EnabledModules::EVENTS;

        Self {
            enabled_modules: modules,
            kernel_debug: KernelTraceModule {},
            costing: CostingModule {
                fee_reserve,
                fee_table,
                max_call_depth: execution_config.max_call_depth,
                payload_len,
                num_of_signatures,
            },
            node_move: NodeMoveModule {},
            auth: AuthModule {
                params: auth_zone_params.clone(),
                auth_zone_stack: Vec::new(),
            },
            logger: LoggerModule::default(),
            transaction_runtime: TransactionRuntimeModule {
                tx_hash,
                next_id: 0,
            },
            transaction_limits: TransactionLimitsModule::new(TransactionLimitsConfig {
                max_wasm_memory: execution_config.max_wasm_mem_per_transaction,
                max_wasm_memory_per_call_frame: execution_config.max_wasm_mem_per_call_frame,
                max_substate_read_count: execution_config.max_substate_reads_per_transaction,
                max_substate_write_count: execution_config.max_substate_writes_per_transaction,
                max_substate_size: execution_config.max_substate_size,
                max_invoke_payload_size: execution_config.max_invoke_input_size,
            }),
            execution_trace: ExecutionTraceModule::new(
                execution_config.execution_trace.unwrap_or(0),
            ),
            events: EventsModule::default(),
            virtualization: VirtualizationModule,
        }
    }
}

//====================================================================
// NOTE: Modules are applied in the reverse order of initialization!
//====================================================================

impl<V: SystemCallbackObject> SystemModule<SystemConfig<V>> for SystemModuleMixer {
    #[trace_resources]
    fn on_init<Y: KernelApi<SystemConfig<V>>>(api: &mut Y) -> Result<(), RuntimeError> {
        let modules: EnabledModules = api.kernel_get_system().modules.enabled_modules;

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
            KernelTraceModule::on_init(api)?;
        }

        // Enable events
        if modules.contains(EnabledModules::EVENTS) {
            EventsModule::on_init(api)?;
        }

        Ok(())
    }

    #[trace_resources]
    fn on_teardown<Y: KernelApi<SystemConfig<V>>>(api: &mut Y) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, on_teardown(api))
    }

    #[trace_resources(log=input_size)]
    fn before_invoke<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        identifier: &KernelInvocation<Actor>,
        input_size: usize,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, before_invoke(api, identifier, input_size))
    }

    #[trace_resources]
    fn before_push_frame<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        callee: &Actor,
        update: &mut CallFrameUpdate,
        args: &IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, before_push_frame(api, callee, update, args))
    }

    #[trace_resources]
    fn on_execution_start<Y: KernelApi<SystemConfig<V>>>(api: &mut Y) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, on_execution_start(api))
    }

    #[trace_resources]
    fn on_execution_finish<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        update: &CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, on_execution_finish(api, update))
    }

    #[trace_resources]
    fn after_pop_frame<Y: KernelApi<SystemConfig<V>>>(api: &mut Y) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, after_pop_frame(api))
    }

    #[trace_resources(log=output_size)]
    fn after_invoke<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        output_size: usize,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, after_invoke(api, output_size))
    }

    #[trace_resources(log=entity_type)]
    fn on_allocate_node_id<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        entity_type: Option<EntityType>,
        virtual_node: bool,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, on_allocate_node_id(api, entity_type, virtual_node))
    }

    #[trace_resources]
    fn before_create_node<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        node_id: &NodeId,
        node_substates: &NodeSubstates,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, before_create_node(api, node_id, node_substates))
    }

    #[trace_resources]
    fn after_create_node<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        node_id: &NodeId,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, after_create_node(api, node_id))
    }

    #[trace_resources]
    fn before_drop_node<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        node_id: &NodeId,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, before_drop_node(api, node_id))
    }

    #[trace_resources]
    fn after_drop_node<Y: KernelApi<SystemConfig<V>>>(api: &mut Y) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, after_drop_node(api))
    }

    #[trace_resources]
    fn before_lock_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        node_id: &NodeId,
        module_id: &PartitionNumber,
        substate_key: &SubstateKey,
        flags: &LockFlags,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api,
            before_lock_substate(api, node_id, module_id, substate_key, flags)
        )
    }

    #[trace_resources(log=size)]
    fn after_lock_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        handle: LockHandle,
        first_lock_from_db: bool,
        size: usize,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api,
            after_lock_substate(api, handle, first_lock_from_db, size)
        )
    }

    #[trace_resources(log=size)]
    fn on_read_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        lock_handle: LockHandle,
        size: usize,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, on_read_substate(api, lock_handle, size))
    }

    #[trace_resources(log=size)]
    fn on_write_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        lock_handle: LockHandle,
        size: usize,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, on_write_substate(api, lock_handle, size))
    }

    #[trace_resources]
    fn on_drop_lock<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        lock_handle: LockHandle,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, on_drop_lock(api, lock_handle))
    }
}
