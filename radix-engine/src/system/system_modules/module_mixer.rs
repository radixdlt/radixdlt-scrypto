use super::costing::{CostingError, ExecutionCostingEntry, FinalizationCostingEntry, StorageType};
use super::limits::TransactionLimitsError;
use crate::errors::*;
use crate::internal_prelude::*;
use crate::kernel::call_frame::CallFrameMessage;
use crate::kernel::kernel_api::KernelInvocation;
use crate::kernel::kernel_api::{KernelApi, KernelInternalApi};
use crate::kernel::kernel_callback_api::{
    CloseSubstateEvent, CreateNodeEvent, DrainSubstatesEvent, DropNodeEvent, MoveModuleEvent,
    OpenSubstateEvent, ReadSubstateEvent, RemoveSubstateEvent, ScanKeysEvent,
    ScanSortedSubstatesEvent, SetSubstateEvent, WriteSubstateEvent,
};
use crate::system::actor::Actor;
use crate::system::module::{InitSystemModule, SystemModule};
use crate::system::system::SystemService;
use crate::system::system_callback::System;
use crate::system::system_callback_api::SystemCallbackObject;
use crate::system::system_modules::auth::AuthModule;
use crate::system::system_modules::costing::CostingModule;
use crate::system::system_modules::costing::FeeTable;
use crate::system::system_modules::costing::SystemLoanFeeReserve;
use crate::system::system_modules::execution_trace::ExecutionTraceModule;
use crate::system::system_modules::kernel_trace::KernelTraceModule;
use crate::system::system_modules::limits::{LimitsModule, TransactionLimitsConfig};
use crate::system::system_modules::transaction_runtime::{Event, TransactionRuntimeModule};
use crate::transaction::{CostingParameters, ExecutionConfig};
use bitflags::bitflags;
use paste::paste;
use radix_common::crypto::Hash;
use radix_engine_interface::api::ModuleId;
use radix_engine_profiling_derive::trace_resources;
use radix_transactions::model::{AuthZoneParams, TransactionCostingParameters};
use crate::track::BootStore;

bitflags! {
    pub struct EnabledModules: u32 {
        // Kernel trace, for debugging only
        const KERNEL_TRACE = 0x1 << 0;

        // Limits, costing and auth
        const LIMITS = 0x01 << 1;
        const COSTING = 0x01 << 2;
        const AUTH = 0x01 << 3;

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
        Self::TRANSACTION_RUNTIME
    }

    pub fn for_system_transaction() -> Self {
        Self::AUTH | Self::TRANSACTION_RUNTIME
    }

    pub fn for_notarized_transaction() -> Self {
        Self::LIMITS | Self::COSTING | Self::AUTH | Self::TRANSACTION_RUNTIME
    }

    pub fn for_test_transaction() -> Self {
        Self::for_notarized_transaction() | Self::KERNEL_TRACE
    }

    pub fn for_preview() -> Self {
        Self::for_notarized_transaction() | Self::EXECUTION_TRACE
    }

    pub fn for_preview_no_auth() -> Self {
        Self::for_preview() - Self::AUTH
    }
}

#[allow(dead_code)]
pub struct SystemModuleMixer {
    // TODO: Use option instead of default for module states?
    // The original reason for performance, but we should double check.

    /* flags */
    pub enabled_modules: EnabledModules,

    /* states */
    pub(super) kernel_trace: KernelTraceModule,
    pub(super) limits: LimitsModule,
    pub(super) costing: CostingModule,
    pub(super) auth: AuthModule,
    pub(crate) transaction_runtime: TransactionRuntimeModule,
    pub(super) execution_trace: ExecutionTraceModule,
}

// Macro generates default modules dispatches call based on passed function name and arguments.
macro_rules! internal_call_dispatch {
    ($system:expr, $fn:ident ( $($param:ident),*) ) => {
        paste! {
        {
            let modules: EnabledModules = $system.modules.enabled_modules;
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
        network_definition: NetworkDefinition,
        tx_hash: Hash,
        auth_zone_params: AuthZoneParams,
        fee_reserve: SystemLoanFeeReserve,
        fee_table: FeeTable,
        payload_len: usize,
        num_of_signature_validations: usize,
        execution_config: &ExecutionConfig,
    ) -> Self {
        Self {
            enabled_modules,
            kernel_trace: KernelTraceModule {},
            costing: CostingModule {
                fee_reserve,
                fee_table,
                max_call_depth: execution_config.max_call_depth,
                tx_payload_len: payload_len,
                tx_num_of_signature_validations: num_of_signature_validations,
                max_per_function_royalty_in_xrd: execution_config.max_per_function_royalty_in_xrd,
                enable_cost_breakdown: execution_config.enable_cost_breakdown,
                execution_cost_breakdown: index_map_new(),
                finalization_cost_breakdown: index_map_new(),
                storage_cost_breakdown: index_map_new(),
                on_apply_cost: Default::default(),
            },
            auth: AuthModule {
                params: auth_zone_params.clone(),
            },
            limits: LimitsModule::new(TransactionLimitsConfig {
                max_heap_substate_total_bytes: execution_config.max_heap_substate_total_bytes,
                max_track_substate_total_bytes: execution_config.max_track_substate_total_bytes,
                max_substate_key_size: execution_config.max_substate_key_size,
                max_substate_value_size: execution_config.max_substate_value_size,
                max_invoke_payload_size: execution_config.max_invoke_input_size,
                max_number_of_logs: execution_config.max_number_of_logs,
                max_number_of_events: execution_config.max_number_of_events,
                max_event_size: execution_config.max_event_size,
                max_log_size: execution_config.max_log_size,
                max_panic_message_size: execution_config.max_panic_message_size,
            }),
            execution_trace: ExecutionTraceModule::new(execution_config.max_execution_trace_depth),
            transaction_runtime: TransactionRuntimeModule {
                network_definition,
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

impl InitSystemModule for SystemModuleMixer {
    #[trace_resources]
    fn init<S: BootStore>(&mut self, store: &S) -> Result<(), BootloadingError> {
        let modules: EnabledModules = self.enabled_modules;

        // Enable execution trace
        if modules.contains(EnabledModules::EXECUTION_TRACE) {
            self.execution_trace.init(store)?;
        }

        // Enable transaction runtime
        if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
            self.transaction_runtime.init(store)?;
        }

        // Enable auth
        if modules.contains(EnabledModules::AUTH) {
            self.auth.init(store)?;
        }

        // Enable costing
        if modules.contains(EnabledModules::COSTING) {
            self.costing.init(store)?;
        }

        // Enable transaction limits
        if modules.contains(EnabledModules::LIMITS) {
            self.limits.init(store)?;
        }

        // Enable kernel trace
        if modules.contains(EnabledModules::KERNEL_TRACE) {
            self.kernel_trace.init(store)?;
        }

        Ok(())
    }
}

impl<V: SystemCallbackObject> SystemModule<System<V>> for SystemModuleMixer {
    #[trace_resources]
    fn on_teardown<Y: KernelApi<System<V>>>(api: &mut Y) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api.kernel_get_system(), on_teardown(api))
    }

    #[trace_resources(log=invocation.len())]
    fn before_invoke<Y: KernelApi<System<V>>>(
        api: &mut Y,
        invocation: &KernelInvocation<Actor>,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api.kernel_get_system(), before_invoke(api, invocation))
    }

    #[trace_resources]
    fn on_execution_start<Y: KernelApi<System<V>>>(api: &mut Y) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api.kernel_get_system(), on_execution_start(api))
    }

    #[trace_resources]
    fn on_execution_finish<Y: KernelApi<System<V>>>(
        api: &mut Y,
        message: &CallFrameMessage,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api.kernel_get_system(), on_execution_finish(api, message))
    }

    #[trace_resources]
    fn after_invoke<Y: KernelApi<System<V>>>(
        api: &mut Y,
        output: &IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api.kernel_get_system(), after_invoke(api, output))
    }

    #[trace_resources(log=entity_type)]
    fn on_allocate_node_id<Y: KernelApi<System<V>>>(
        api: &mut Y,
        entity_type: EntityType,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api.kernel_get_system(),
            on_allocate_node_id(api, entity_type)
        )
    }

    #[trace_resources]
    fn on_create_node<Y: KernelInternalApi<System<V>>>(
        api: &mut Y,
        event: &CreateNodeEvent,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api.kernel_get_system(), on_create_node(api, event))
    }

    #[trace_resources]
    fn on_pin_node(system: &mut System<V>, node_id: &NodeId) -> Result<(), RuntimeError> {
        internal_call_dispatch!(system, on_pin_node(system, node_id))
    }

    #[trace_resources]
    fn on_drop_node<Y: KernelInternalApi<System<V>>>(
        api: &mut Y,
        event: &DropNodeEvent,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api.kernel_get_system(), on_drop_node(api, event))
    }

    #[trace_resources]
    fn on_move_module<Y: KernelInternalApi<System<V>>>(
        api: &mut Y,
        event: &MoveModuleEvent,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api.kernel_get_system(), on_move_module(api, event))
    }

    #[trace_resources]
    fn on_open_substate<Y: KernelInternalApi<System<V>>>(
        api: &mut Y,
        event: &OpenSubstateEvent,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api.kernel_get_system(), on_open_substate(api, event))
    }

    #[trace_resources]
    fn on_mark_substate_as_transient(
        system: &mut System<V>,
        node_id: &NodeId,
        partition_number: &PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            system,
            on_mark_substate_as_transient(system, node_id, partition_number, substate_key)
        )
    }

    #[trace_resources(log=event.is_about_heap())]
    fn on_read_substate<Y: KernelInternalApi<System<V>>>(
        api: &mut Y,
        event: &ReadSubstateEvent,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api.kernel_get_system(), on_read_substate(api, event))
    }

    #[trace_resources]
    fn on_write_substate<Y: KernelInternalApi<System<V>>>(
        api: &mut Y,
        event: &WriteSubstateEvent,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api.kernel_get_system(), on_write_substate(api, event))
    }

    #[trace_resources]
    fn on_close_substate<Y: KernelInternalApi<System<V>>>(
        api: &mut Y,
        event: &CloseSubstateEvent,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(api.kernel_get_system(), on_close_substate(api, event))
    }

    #[trace_resources]
    fn on_set_substate(
        system: &mut System<V>,
        event: &SetSubstateEvent,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(system, on_set_substate(system, event))
    }

    #[trace_resources]
    fn on_remove_substate(
        system: &mut System<V>,
        event: &RemoveSubstateEvent,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(system, on_remove_substate(system, event))
    }

    #[trace_resources]
    fn on_scan_keys(
        system: &mut System<V>,
        event: &ScanKeysEvent,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(system, on_scan_keys(system, event))
    }

    #[trace_resources]
    fn on_drain_substates(
        system: &mut System<V>,
        event: &DrainSubstatesEvent,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(system, on_drain_substates(system, event))
    }

    #[trace_resources]
    fn on_scan_sorted_substates(
        system: &mut System<V>,
        event: &ScanSortedSubstatesEvent,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(system, on_scan_sorted_substates(system, event))
    }
}

impl SystemModuleMixer {
    // Note that module mixer is called by both kernel and system.
    // - Kernel uses the `SystemModule<SystemConfig<V>>` trait above;
    // - System uses methods defined below (TODO: add a trait?)

    pub fn on_call_method<Y, V>(
        api: &mut SystemService<Y, V>,
        receiver: &NodeId,
        module_id: ModuleId,
        direct_access: bool,
        ident: &str,
        args: &IndexedScryptoValue,
    ) -> Result<NodeId, RuntimeError>
    where
        V: SystemCallbackObject,
        Y: KernelApi<System<V>>,
    {
        let auth_zone = if api
            .kernel_get_system_state()
            .system
            .modules
            .enabled_modules
            .contains(EnabledModules::AUTH)
        {
            AuthModule::on_call_method(api, receiver, module_id, direct_access, ident, args)?
        } else {
            AuthModule::on_call_fn_mock(
                api,
                Some((receiver, direct_access)),
                btreeset!(),
                btreeset!(),
            )?
        };

        Ok(auth_zone)
    }

    pub fn on_call_method_finish<Y, V>(
        api: &mut SystemService<Y, V>,
        auth_zone: NodeId,
    ) -> Result<(), RuntimeError>
    where
        V: SystemCallbackObject,
        Y: KernelApi<System<V>>,
    {
        AuthModule::on_call_method_finish(api, auth_zone)
    }

    pub fn on_call_function<V, Y>(
        api: &mut SystemService<Y, V>,
        blueprint_id: &BlueprintId,
        ident: &str,
    ) -> Result<NodeId, RuntimeError>
    where
        V: SystemCallbackObject,
        Y: KernelApi<System<V>>,
    {
        let auth_zone = if api
            .kernel_get_system_state()
            .system
            .modules
            .enabled_modules
            .contains(EnabledModules::AUTH)
        {
            AuthModule::on_call_function(api, blueprint_id, ident)?
        } else {
            AuthModule::on_call_fn_mock(api, None, btreeset!(), btreeset!())?
        };

        Ok(auth_zone)
    }

    pub fn on_call_function_finish<V, Y>(
        api: &mut SystemService<Y, V>,
        auth_zone: NodeId,
    ) -> Result<(), RuntimeError>
    where
        V: SystemCallbackObject,
        Y: KernelApi<System<V>>,
    {
        AuthModule::on_call_function_finish(api, auth_zone)
    }

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

    pub fn assert_can_add_event(&mut self) -> Result<(), RuntimeError> {
        if self.enabled_modules.contains(EnabledModules::LIMITS) {
            if self.transaction_runtime.events.len() >= self.limits.config().max_number_of_events {
                return Err(RuntimeError::SystemModuleError(
                    SystemModuleError::TransactionLimitsError(
                        TransactionLimitsError::TooManyEvents,
                    ),
                ));
            }
        }

        Ok(())
    }

    pub fn add_event_unchecked(&mut self, event: Event) -> Result<(), RuntimeError> {
        if self.enabled_modules.contains(EnabledModules::LIMITS) {
            if event.payload.len() > self.limits.config().max_event_size {
                return Err(RuntimeError::SystemModuleError(
                    SystemModuleError::TransactionLimitsError(
                        TransactionLimitsError::EventSizeTooLarge {
                            actual: event.payload.len(),
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
            self.transaction_runtime.add_event(event);
        }

        Ok(())
    }

    pub fn checked_add_event(&mut self, event: Event) -> Result<(), RuntimeError> {
        self.assert_can_add_event()?;
        self.add_event_unchecked(event)?;
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

    pub fn add_replacement(&mut self, old: (NodeId, ModuleId), new: (NodeId, ModuleId)) {
        if self
            .enabled_modules
            .contains(EnabledModules::TRANSACTION_RUNTIME)
        {
            self.transaction_runtime.add_replacement(old, new)
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

    pub fn costing_mut(&mut self) -> Option<&mut CostingModule> {
        if self.enabled_modules.contains(EnabledModules::COSTING) {
            Some(&mut self.costing)
        } else {
            None
        }
    }

    pub fn limits_mut(&mut self) -> Option<&mut LimitsModule> {
        if self.enabled_modules.contains(EnabledModules::LIMITS) {
            Some(&mut self.limits)
        } else {
            None
        }
    }

    pub fn transaction_runtime(&mut self) -> Option<&TransactionRuntimeModule> {
        if self
            .enabled_modules
            .contains(EnabledModules::TRANSACTION_RUNTIME)
        {
            Some(&self.transaction_runtime)
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
        costing_entry: ExecutionCostingEntry,
    ) -> Result<(), RuntimeError> {
        if self.enabled_modules.contains(EnabledModules::COSTING) {
            self.costing
                .apply_execution_cost(costing_entry)
                .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))
        } else {
            Ok(())
        }
    }

    pub fn apply_finalization_cost(
        &mut self,
        costing_entry: FinalizationCostingEntry,
    ) -> Result<(), CostingError> {
        if self.enabled_modules.contains(EnabledModules::COSTING) {
            self.costing.apply_finalization_cost(costing_entry)
        } else {
            Ok(())
        }
    }

    pub fn apply_storage_cost(
        &mut self,
        storage_type: StorageType,
        size_increase: usize,
    ) -> Result<(), CostingError> {
        if self.enabled_modules.contains(EnabledModules::COSTING) {
            self.costing.apply_storage_cost(storage_type, size_increase)
        } else {
            Ok(())
        }
    }

    pub fn lock_fee(
        &mut self,
        vault_id: NodeId,
        locked_fee: LiquidFungibleResource,
        contingent: bool,
    ) {
        if self.enabled_modules.contains(EnabledModules::COSTING) {
            self.costing.lock_fee(vault_id, locked_fee, contingent);
        } else {
            panic!("Fungible Vault Application layer should prevent call to credit if costing not enabled");
        }
    }

    pub fn events(&self) -> &Vec<Event> {
        &self.transaction_runtime.events
    }

    pub fn logs(&self) -> &Vec<(Level, String)> {
        &self.transaction_runtime.logs
    }
}
