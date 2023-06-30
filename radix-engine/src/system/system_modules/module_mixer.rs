use super::costing::CostingEntry;
use super::limits::TransactionLimitsError;
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
use crate::system::system_modules::execution_trace::ExecutionTraceModule;
use crate::system::system_modules::kernel_trace::KernelTraceModule;
use crate::system::system_modules::limits::{LimitsModule, TransactionLimitsConfig};
use crate::system::system_modules::node_move::NodeMoveModule;
use crate::system::system_modules::transaction_runtime::TransactionRuntimeModule;
use crate::track::interface::StoreCommit;
use crate::track::interface::{NodeSubstates, StoreAccessInfo};
use crate::transaction::ExecutionConfig;
use crate::types::*;
use bitflags::bitflags;
use paste::paste;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::crypto::Hash;
use resources_tracker_macro::trace_resources;
use transaction::model::AuthZoneParams;

bitflags! {
    pub struct EnabledModules: u32 {
        // Kernel trace, for debugging only
        const KERNEL_TRACE = 0x1 << 0;

        // Limits, costing and auth
        const LIMITS = 0x01 << 1;
        const COSTING = 0x01 << 2;
        const AUTH = 0x01 << 3;
        const NODE_MOVE = 0x01 << 4;

        // Transaction runtime data
        const TRANSACTION_RUNTIME = 0x01 << 5;

        // Execution trace, for preview only
        const EXECUTION_TRACE = 0x01 << 6;
    }
}

impl EnabledModules {
    /// The difference between genesis transaction and system transaction is "no auth".
    /// TODO: double check if this is the right assumption.
    pub fn for_genesis_transaction() -> Self {
        Self::LIMITS | Self::NODE_MOVE | Self::TRANSACTION_RUNTIME
    }

    pub fn for_system_transaction() -> Self {
        Self::LIMITS | Self::AUTH | Self::NODE_MOVE | Self::TRANSACTION_RUNTIME
    }

    pub fn for_notarized_transaction() -> Self {
        Self::LIMITS | Self::COSTING | Self::AUTH | Self::NODE_MOVE | Self::TRANSACTION_RUNTIME
    }

    pub fn for_test_transaction() -> Self {
        Self::for_notarized_transaction() | Self::KERNEL_TRACE
    }

    pub fn for_preview() -> Self {
        Self::for_notarized_transaction() | Self::EXECUTION_TRACE
    }
}

#[allow(dead_code)]
pub struct SystemModuleMixer {
    // TODO: Use option instead of default for module states?
    // The original reason for performance, but we should double check.

    /* flags */
    enabled_modules: EnabledModules,

    /* states */
    pub(super) kernel_trace: KernelTraceModule,
    pub(super) limits: LimitsModule,
    pub(super) costing: CostingModule,
    pub(super) auth: AuthModule,
    pub(super) node_move: NodeMoveModule,
    pub(super) transaction_runtime: TransactionRuntimeModule,
    pub(super) execution_trace: ExecutionTraceModule,
}

// Macro generates default modules dispatches call based on passed function name and arguments.
macro_rules! internal_call_dispatch {
    ($api:ident, $fn:ident ( $($param:ident),*) ) => {
        paste! {
        {
            let modules: EnabledModules = $api.kernel_get_system().modules.enabled_modules;
            if modules.contains(EnabledModules::KERNEL_TRACE) {
                KernelTraceModule::[< $fn >]($($param, )*)?;
            }
            if modules.contains(EnabledModules::LIMITS) {
                 LimitsModule::[< $fn >]($($param, )*)?;
            }
            if modules.contains(EnabledModules::COSTING) {
                CostingModule::[< $fn >]($($param, )*)?;
            }
            if modules.contains(EnabledModules::AUTH) {
                AuthModule::[< $fn >]($($param, )*)?;
            }
            if modules.contains(EnabledModules::NODE_MOVE) {
                NodeMoveModule::[< $fn >]($($param, )*)?;
            }
            if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
                TransactionRuntimeModule::[< $fn >]($($param, )*)?;
            }
            if modules.contains(EnabledModules::EXECUTION_TRACE) {
                ExecutionTraceModule::[< $fn >]($($param, )*)?;
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
            kernel_trace: KernelTraceModule {},
            costing: CostingModule {
                fee_reserve,
                fee_table,
                max_call_depth: execution_config.max_call_depth,
                payload_len,
                num_of_signatures,
                max_per_function_royalty_in_xrd: execution_config.max_per_function_royalty_in_xrd,
                enable_cost_breakdown: execution_config.enable_cost_breakdown,
                costing_traces: index_map_new(),
            },
            node_move: NodeMoveModule {},
            auth: AuthModule {
                params: auth_zone_params.clone(),
                auth_zone_stack: Vec::new(),
            },
            limits: LimitsModule::new(TransactionLimitsConfig {
                max_number_of_substates_in_track: execution_config.max_number_of_substates_in_track,
                max_number_of_substates_in_heap: execution_config.max_number_of_substates_in_heap,
                max_substate_size: execution_config.max_substate_size,
                max_invoke_payload_size: execution_config.max_invoke_input_size,
                max_number_of_logs: execution_config.max_number_of_logs,
                max_number_of_events: execution_config.max_number_of_events,
                max_event_size: execution_config.max_event_size,
                max_log_size: execution_config.max_log_size,
                max_panic_message_size: execution_config.max_panic_message_size,
            }),
            execution_trace: ExecutionTraceModule::new(execution_config.max_execution_trace_depth),
            transaction_runtime: TransactionRuntimeModule {
                tx_hash,
                next_id: 0,
                logs: Vec::new(),
                events: Vec::new(),
                replacements: index_map_new(),
            },
        }
    }

    pub fn unpack(
        self,
    ) -> (
        CostingModule,
        TransactionRuntimeModule,
        ExecutionTraceModule,
    ) {
        (self.costing, self.transaction_runtime, self.execution_trace)
    }
}

//====================================================================
// NOTE: Modules are applied in the reverse order of initialization!
// This has an impact if there is module dependency.
//====================================================================

impl<V: SystemCallbackObject> SystemModule<SystemConfig<V>> for SystemModuleMixer {
    #[trace_resources]
    fn on_init<Y: KernelApi<SystemConfig<V>>>(api: &mut Y) -> Result<(), RuntimeError> {
        let modules: EnabledModules = api.kernel_get_system().modules.enabled_modules;

        // Enable execution trace
        if modules.contains(EnabledModules::EXECUTION_TRACE) {
            ExecutionTraceModule::on_init(api)?;
        }

        // Enable transaction runtime
        if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
            TransactionRuntimeModule::on_init(api)?;
        }

        // Enable node move
        if modules.contains(EnabledModules::NODE_MOVE) {
            NodeMoveModule::on_init(api)?;
        }

        // Enable auth
        if modules.contains(EnabledModules::AUTH) {
            AuthModule::on_init(api)?;
        }

        // Enable costing
        if modules.contains(EnabledModules::COSTING) {
            CostingModule::on_init(api)?;
        }

        // Enable transaction limits
        if modules.contains(EnabledModules::LIMITS) {
            LimitsModule::on_init(api)?;
        }

        // Enable kernel trace
        if modules.contains(EnabledModules::KERNEL_TRACE) {
            KernelTraceModule::on_init(api)?;
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
        total_substate_size: usize,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api,
            after_create_node(api, node_id, total_substate_size, store_access)
        )
    }

    #[trace_resources]
    fn after_move_modules<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        src_node_id: &NodeId,
        dest_node_id: &NodeId,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api,
            after_move_modules(api, src_node_id, dest_node_id, store_access)
        )
    }

    #[trace_resources]
    fn before_drop_node<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        node_id: &NodeId,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, before_drop_node(api, node_id))
    }

    #[trace_resources]
    fn after_drop_node<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        total_substate_size: usize,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, after_drop_node(api, total_substate_size))
    }

    #[trace_resources]
    fn before_open_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        node_id: &NodeId,
        partition_number: &PartitionNumber,
        substate_key: &SubstateKey,
        flags: &LockFlags,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api,
            before_open_substate(api, node_id, partition_number, substate_key, flags)
        )
    }

    #[trace_resources(log=size)]
    fn after_open_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        handle: LockHandle,
        node_id: &NodeId,
        store_access: &StoreAccessInfo,
        size: usize,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api,
            after_open_substate(api, handle, node_id, store_access, size)
        )
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
    fn on_close_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        lock_handle: LockHandle,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, on_close_substate(api, lock_handle, store_access))
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
        value_size: usize,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, on_set_substate(api, value_size, store_access))
    }

    #[trace_resources]
    fn on_take_substates<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api, on_take_substates(api, store_access))
    }
}

impl SystemModuleMixer {
    // Note that module mixer is called by both kernel and system.
    // - Kernel uses the `SystemModule<SystemConfig<V>>` trait above;
    // - System uses methods defined below (TODO: add a trait?)

    pub fn add_log(&mut self, level: Level, message: String) -> Result<(), RuntimeError> {
        if self.enabled_modules.contains(EnabledModules::LIMITS) {
            if self.transaction_runtime.logs.len() >= self.limits.config().max_number_of_logs {
                return Err(RuntimeError::SystemModuleError(
                    SystemModuleError::TransactionLimitsError(TransactionLimitsError::TooManyLogs),
                ));
            }
            if message.len() > self.limits.config().max_log_size {
                return Err(RuntimeError::SystemModuleError(
                    SystemModuleError::TransactionLimitsError(
                        TransactionLimitsError::LogSizeTooLarge {
                            actual: message.len(),
                            max: self.limits.config().max_log_size,
                        },
                    ),
                ));
            }
        }

        if self
            .enabled_modules
            .contains(EnabledModules::TRANSACTION_RUNTIME)
        {
            self.transaction_runtime.add_log(level, message);
        }

        Ok(())
    }

    pub fn add_event(
        &mut self,
        identifier: EventTypeIdentifier,
        data: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        if self.enabled_modules.contains(EnabledModules::LIMITS) {
            if self.transaction_runtime.events.len() >= self.limits.config().max_number_of_events {
                return Err(RuntimeError::SystemModuleError(
                    SystemModuleError::TransactionLimitsError(
                        TransactionLimitsError::TooManyEvents,
                    ),
                ));
            }
            if data.len() > self.limits.config().max_event_size {
                return Err(RuntimeError::SystemModuleError(
                    SystemModuleError::TransactionLimitsError(
                        TransactionLimitsError::EventSizeTooLarge {
                            actual: data.len(),
                            max: self.limits.config().max_event_size,
                        },
                    ),
                ));
            }
        }

        if self
            .enabled_modules
            .contains(EnabledModules::TRANSACTION_RUNTIME)
        {
            self.transaction_runtime.add_event(identifier, data)
        }

        Ok(())
    }

    pub fn set_panic_message(&mut self, message: String) -> Result<(), RuntimeError> {
        if self.enabled_modules.contains(EnabledModules::LIMITS) {
            if message.len() > self.limits.config().max_panic_message_size {
                return Err(RuntimeError::SystemModuleError(
                    SystemModuleError::TransactionLimitsError(
                        TransactionLimitsError::PanicMessageSizeTooLarge {
                            actual: message.len(),
                            max: self.limits.config().max_panic_message_size,
                        },
                    ),
                ));
            }
        }

        Ok(())
    }

    pub fn add_replacement(
        &mut self,
        old: (NodeId, ObjectModuleId),
        new: (NodeId, ObjectModuleId),
    ) {
        if self
            .enabled_modules
            .contains(EnabledModules::TRANSACTION_RUNTIME)
        {
            self.transaction_runtime.add_replacement(old, new)
        }
    }

    pub fn auth_zone_id(&mut self) -> Option<NodeId> {
        if self.enabled_modules.contains(EnabledModules::AUTH) {
            self.auth.last_auth_zone()
        } else {
            None
        }
    }

    pub fn fee_reserve(&mut self) -> Option<&SystemLoanFeeReserve> {
        if self.enabled_modules.contains(EnabledModules::COSTING) {
            Some(&self.costing.fee_reserve)
        } else {
            None
        }
    }

    pub fn costing(&mut self) -> Option<&CostingModule> {
        if self.enabled_modules.contains(EnabledModules::COSTING) {
            Some(&self.costing)
        } else {
            None
        }
    }

    pub fn transaction_hash(&self) -> Option<Hash> {
        if self
            .enabled_modules
            .contains(EnabledModules::TRANSACTION_RUNTIME)
        {
            Some(self.transaction_runtime.tx_hash)
        } else {
            None
        }
    }

    pub fn generate_ruid(&mut self) -> Option<[u8; 32]> {
        if self
            .enabled_modules
            .contains(EnabledModules::TRANSACTION_RUNTIME)
        {
            Some(self.transaction_runtime.generate_ruid())
        } else {
            None
        }
    }

    pub fn update_instruction_index(&mut self, new_index: usize) {
        if self
            .enabled_modules
            .contains(EnabledModules::EXECUTION_TRACE)
        {
            self.execution_trace.update_instruction_index(new_index)
        }
    }

    pub fn apply_execution_cost(
        &mut self,
        costing_entry: CostingEntry,
    ) -> Result<(), RuntimeError> {
        if self.enabled_modules.contains(EnabledModules::COSTING) {
            self.costing.apply_execution_cost(costing_entry)
        } else {
            Ok(())
        }
    }

    pub fn apply_state_expansion_cost(
        &mut self,
        store_commit: &StoreCommit,
    ) -> Result<(), RuntimeError> {
        if self.enabled_modules.contains(EnabledModules::COSTING) {
            self.costing.apply_state_expansion_cost(store_commit)
        } else {
            Ok(())
        }
    }

    pub fn credit_cost_units(
        &mut self,
        vault_id: NodeId,
        locked_fee: LiquidFungibleResource,
        contingent: bool,
    ) -> Result<LiquidFungibleResource, RuntimeError> {
        if self.enabled_modules.contains(EnabledModules::COSTING) {
            self.costing
                .credit_cost_units(vault_id, locked_fee, contingent)
        } else {
            Ok(locked_fee)
        }
    }
}
