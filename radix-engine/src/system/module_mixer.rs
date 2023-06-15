use crate::errors::*;
use crate::kernel::actor::Actor;
use crate::kernel::call_frame::Message;
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
use crate::system::system_modules::kernel_trace::KernelDebugModule;
use crate::system::system_modules::logger::LoggerModule;
use crate::system::system_modules::node_move::NodeMoveModule;
use crate::system::system_modules::transaction_limits::{
    TransactionLimitsConfig, TransactionLimitsModule,
};
use crate::system::system_modules::transaction_runtime::TransactionRuntimeModule;
use crate::system::system_modules::virtualization::VirtualizationModule;
use crate::track::interface::{NodeSubstates, StoreAccessInfo};
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
        const KERNEL_TRACE = 0x1 << 0;
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

impl EnabledModules {
    /// The difference between genesis transaction and system transaction is "no auth".
    /// TODO: double check if this is the right assumption.
    pub fn for_genesis_transaction() -> Self {
        Self::NODE_MOVE
            | Self::LOGGER
            | Self::TRANSACTION_RUNTIME
            | Self::TRANSACTION_LIMITS
            | Self::EVENTS
    }

    pub fn for_system_transaction() -> Self {
        Self::NODE_MOVE
            | Self::AUTH
            | Self::LOGGER
            | Self::TRANSACTION_RUNTIME
            | Self::TRANSACTION_LIMITS
            | Self::EVENTS
    }

    pub fn for_notarized_transaction() -> Self {
        Self::COSTING
            | Self::NODE_MOVE
            | Self::AUTH
            | Self::LOGGER
            | Self::TRANSACTION_RUNTIME
            | Self::TRANSACTION_LIMITS
            | Self::EVENTS
    }

    pub fn for_test_transaction() -> Self {
        Self::for_notarized_transaction() | Self::KERNEL_TRACE
    }

    pub fn for_preview() -> Self {
        Self::for_notarized_transaction() | Self::EXECUTION_TRACE
    }
}

// TODO: use option instead of defaults?
#[allow(dead_code)]
pub struct SystemModuleMixer {
    /* flags */
    // TODO: check if the original assumption is still true.
    // The reason for using bit flags, rather than Option<T>, was to improve method dispatching performance.
    enabled_modules: EnabledModules,

    /* states */
    kernel_trace: KernelDebugModule,
    costing: CostingModule,
    node_move: NodeMoveModule,
    auth: AuthModule,
    logger: LoggerModule,
    transaction_runtime: TransactionRuntimeModule,
    execution_trace: ExecutionTraceModule,
    transaction_limits: TransactionLimitsModule,
    events: EventsModule,
    virtualization: VirtualizationModule,
}

// Macro generates default modules dispatches call based on passed function name and arguments.
macro_rules! internal_call_dispatch {
    ($api:ident, $fn:ident ( $($param:ident),*) ) => {
        paste! {
        {
            let modules: EnabledModules = $api.kernel_get_system().modules.enabled_modules;
            if modules.contains(EnabledModules::KERNEL_TRACE) {
                KernelDebugModule::[< $fn >]($($param, )*)?;
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
    pub fn new(
        enabled_modules: EnabledModules,
        tx_hash: Hash,
        auth_zone_params: AuthZoneParams,
        fee_reserve: SystemLoanFeeReserve,
        fee_table: FeeTable,
        payload_len: usize,
        num_of_signatures: usize,
        execution_config: &ExecutionConfig,
    ) -> Self {
        Self {
            enabled_modules,
            kernel_trace: KernelDebugModule {},
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
            execution_trace: ExecutionTraceModule::new(execution_config.max_execution_trace_depth),
            events: EventsModule::default(),
            virtualization: VirtualizationModule,
        }
    }

    pub fn costing_module(&mut self) -> Option<&mut CostingModule> {
        if self.enabled_modules.contains(EnabledModules::COSTING) {
            Some(&mut self.costing)
        } else {
            None
        }
    }

    pub fn events_module(&mut self) -> Option<&mut EventsModule> {
        if self.enabled_modules.contains(EnabledModules::EVENTS) {
            Some(&mut self.events)
        } else {
            None
        }
    }

    pub fn logger_module(&mut self) -> Option<&mut LoggerModule> {
        if self.enabled_modules.contains(EnabledModules::LOGGER) {
            Some(&mut self.logger)
        } else {
            None
        }
    }

    pub fn auth_module(&mut self) -> Option<&mut AuthModule> {
        if self.enabled_modules.contains(EnabledModules::AUTH) {
            Some(&mut self.auth)
        } else {
            None
        }
    }

    pub fn execution_trace_module(&mut self) -> Option<&mut ExecutionTraceModule> {
        if self
            .enabled_modules
            .contains(EnabledModules::EXECUTION_TRACE)
        {
            Some(&mut self.execution_trace)
        } else {
            None
        }
    }

    pub fn transaction_runtime_module(&mut self) -> Option<&mut TransactionRuntimeModule> {
        if self
            .enabled_modules
            .contains(EnabledModules::TRANSACTION_RUNTIME)
        {
            Some(&mut self.transaction_runtime)
        } else {
            None
        }
    }

    pub fn transaction_limits_module(&mut self) -> Option<&mut TransactionLimitsModule> {
        if self
            .enabled_modules
            .contains(EnabledModules::TRANSACTION_LIMITS)
        {
            Some(&mut self.transaction_limits)
        } else {
            None
        }
    }

    pub fn unpack(
        self,
    ) -> (
        CostingModule,
        EventsModule,
        LoggerModule,
        ExecutionTraceModule,
        TransactionLimitsModule,
    ) {
        (
            self.costing,
            self.events,
            self.logger,
            self.execution_trace,
            self.transaction_limits,
        )
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
        if modules.contains(EnabledModules::KERNEL_TRACE) {
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
        if modules.contains(EnabledModules::KERNEL_TRACE) {
            KernelDebugModule::on_init(api)?;
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

    #[trace_resources(log=invocation.len())]
    fn before_invoke<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        invocation: &KernelInvocation,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, before_invoke(api, invocation))
    }

    #[trace_resources]
    fn before_push_frame<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        callee: &Actor,
        update: &mut Message,
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
        update: &Message,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, on_execution_finish(api, update))
    }

    #[trace_resources]
    fn after_pop_frame<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        dropped_actor: &Actor,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, after_pop_frame(api, dropped_actor))
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
        entity_type: EntityType,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, on_allocate_node_id(api, entity_type))
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
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, after_create_node(api, node_id, store_access))
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
        partition_number: &PartitionNumber,
        substate_key: &SubstateKey,
        flags: &LockFlags,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api,
            before_lock_substate(api, node_id, partition_number, substate_key, flags)
        )
    }

    #[trace_resources(log=size)]
    fn after_lock_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        handle: LockHandle,
        store_access: &StoreAccessInfo,
        size: usize,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, after_lock_substate(api, handle, store_access, size))
    }

    #[trace_resources(log=value_size)]
    fn on_read_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        lock_handle: LockHandle,
        value_size: usize,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api,
            on_read_substate(api, lock_handle, value_size, store_access)
        )
    }

    #[trace_resources(log=value_size)]
    fn on_write_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        lock_handle: LockHandle,
        value_size: usize,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api,
            on_write_substate(api, lock_handle, value_size, store_access)
        )
    }

    #[trace_resources]
    fn on_drop_lock<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        lock_handle: LockHandle,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, on_drop_lock(api, lock_handle, store_access))
    }

    #[trace_resources]
    fn on_scan_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, on_scan_substate(api, store_access))
    }

    #[trace_resources]
    fn on_set_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, on_set_substate(api, store_access))
    }

    #[trace_resources]
    fn on_take_substates<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, on_take_substates(api, store_access))
    }
}
