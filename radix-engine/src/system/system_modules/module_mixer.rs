use super::costing::{CostingError, ExecutionCostingEntry, FinalizationCostingEntry, StorageType};
use super::limits::TransactionLimitsError;
use crate::errors::*;
use crate::internal_prelude::*;
use crate::kernel::call_frame::CallFrameMessage;
use crate::kernel::kernel_api::*;
use crate::kernel::kernel_callback_api::*;
use crate::system::actor::Actor;
use crate::system::module::PrivilegedSystemModule;
use crate::system::module::{InitSystemModule, SystemModule};
use crate::system::system::SystemService;
use crate::system::system_callback::*;
use crate::system::system_modules::auth::AuthModule;
use crate::system::system_modules::costing::CostingModule;
use crate::system::system_modules::costing::SystemLoanFeeReserve;
use crate::system::system_modules::execution_trace::ExecutionTraceModule;
use crate::system::system_modules::kernel_trace::KernelTraceModule;
use crate::system::system_modules::limits::LimitsModule;
use crate::system::system_modules::transaction_runtime::{Event, TransactionRuntimeModule};
use bitflags::bitflags;
use paste::paste;
use radix_common::crypto::Hash;
use radix_engine_interface::api::ModuleId;
use radix_engine_profiling_derive::trace_resources;

bitflags! {
    pub struct EnabledModules: u32 {
        // Kernel trace, for debugging only
        const KERNEL_TRACE = 0x1 << 0;

        // Limits, costing and auth
        const LIMITS = 0x01 << 1;
        const COSTING = 0x01 << 2;
        const AUTH = 0x01 << 3;
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
    (
        $system:expr,
        $fn:ident ( $($param:expr),* )
        $(, $privileged_fn:ident ( $($privileged_fn_param:expr),* ))?
    ) => {
        paste! {{
            let modules: EnabledModules = $system.modules.enabled_modules;
            if modules.contains(EnabledModules::KERNEL_TRACE) {
                KernelTraceModule::[< $fn >]($($param, )*)?;
                $(KernelTraceModule::[< $privileged_fn >]($($privileged_fn_param, )*)?;)?
            }
            if modules.contains(EnabledModules::LIMITS) {
                LimitsModule::[< $fn >]($($param, )*)?;
                $(LimitsModule::[< $privileged_fn >]($($privileged_fn_param, )*)?;)?
            }
            if modules.contains(EnabledModules::COSTING) {
                CostingModule::[< $fn >]($($param, )*)?;
                $(CostingModule::[< $privileged_fn >]($($privileged_fn_param, )*)?;)?
            }
            if modules.contains(EnabledModules::AUTH) {
                AuthModule::[< $fn >]($($param, )*)?;
                $(AuthModule::[< $privileged_fn >]($($privileged_fn_param, )*)?;)?
            }
            if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
                TransactionRuntimeModule::[< $fn >]($($param, )*)?;
                $(TransactionRuntimeModule::[< $privileged_fn >]($($privileged_fn_param, )*)?;)?
            }
            if modules.contains(EnabledModules::EXECUTION_TRACE) {
                ExecutionTraceModule::[< $fn >]($($param, )*)?;
                $(ExecutionTraceModule::[< $privileged_fn >]($($privileged_fn_param, )*)?;)?
            }
            Ok(())
        }}
    };
}

impl SystemModuleMixer {
    pub fn new(
        enabled_modules: EnabledModules,
        kernel_trace: KernelTraceModule,
        transaction_runtime: TransactionRuntimeModule,
        auth: AuthModule,
        limits: LimitsModule,
        costing: CostingModule,
        execution_trace: ExecutionTraceModule,
    ) -> Self {
        Self {
            enabled_modules,
            kernel_trace,
            transaction_runtime,
            auth,
            costing,
            limits,
            execution_trace,
        }
    }

    #[inline]
    pub fn is_kernel_trace_enabled(&self) -> bool {
        self.enabled_modules.contains(EnabledModules::KERNEL_TRACE)
    }

    #[inline]
    pub fn is_execution_trace_enabled(&self) -> bool {
        self.enabled_modules
            .contains(EnabledModules::EXECUTION_TRACE)
    }

    pub fn unpack_costing(self) -> CostingModule {
        self.costing
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
    fn init(&mut self) -> Result<(), BootloadingError> {
        let modules: EnabledModules = self.enabled_modules;

        // Enable execution trace
        if modules.contains(EnabledModules::EXECUTION_TRACE) {
            self.execution_trace.init()?;
        }

        // Enable transaction runtime
        if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
            self.transaction_runtime.init()?;
        }

        // Enable auth
        if modules.contains(EnabledModules::AUTH) {
            self.auth.init()?;
        }

        // Enable costing
        if modules.contains(EnabledModules::COSTING) {
            self.costing.init()?;
        }

        // Enable transaction limits
        if modules.contains(EnabledModules::LIMITS) {
            self.limits.init()?;
        }

        // Enable kernel trace
        if modules.contains(EnabledModules::KERNEL_TRACE) {
            self.kernel_trace.init()?;
        }

        Ok(())
    }

    #[trace_resources]
    fn on_teardown(&mut self) -> Result<(), RuntimeError> {
        let modules: EnabledModules = self.enabled_modules;
        if modules.contains(EnabledModules::KERNEL_TRACE) {
            self.kernel_trace.on_teardown()?;
        }
        if modules.contains(EnabledModules::LIMITS) {
            self.limits.on_teardown()?;
        }
        if modules.contains(EnabledModules::COSTING) {
            self.costing.on_teardown()?;
        }
        if modules.contains(EnabledModules::AUTH) {
            self.auth.on_teardown()?;
        }
        if modules.contains(EnabledModules::TRANSACTION_RUNTIME) {
            self.transaction_runtime.on_teardown()?;
        }
        if modules.contains(EnabledModules::EXECUTION_TRACE) {
            self.execution_trace.on_teardown()?;
        }

        Ok(())
    }
}

impl SystemModuleMixer {
    #[trace_resources(log=invocation.len())]
    pub fn before_invoke(
        api: &mut impl SystemBasedKernelApi,
        invocation: &KernelInvocation<Actor>,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api.kernel_get_system(),
            before_invoke(&mut api.system_module_api(), invocation),
            privileged_before_invoke(api, invocation)
        )
    }

    #[trace_resources]
    pub fn on_execution_start(
        api: &mut impl SystemBasedKernelInternalApi,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api.kernel_get_system(),
            on_execution_start(&mut api.system_module_api())
        )
    }

    #[trace_resources]
    pub fn on_execution_finish(
        api: &mut impl SystemBasedKernelInternalApi,
        message: &CallFrameMessage,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api.kernel_get_system(),
            on_execution_finish(&mut api.system_module_api(), message)
        )
    }

    #[trace_resources]
    pub fn after_invoke(
        api: &mut impl SystemBasedKernelInternalApi,
        output: &IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api.kernel_get_system(),
            after_invoke(&mut api.system_module_api(), output)
        )
    }

    #[trace_resources]
    pub fn on_pin_node(
        api: &mut impl SystemBasedKernelInternalApi,
        node_id: &NodeId,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api.kernel_get_system(),
            on_pin_node(&mut api.system_module_api(), node_id)
        )
    }

    #[trace_resources(log=entity_type)]
    pub fn on_allocate_node_id(
        api: &mut impl SystemBasedKernelInternalApi,
        entity_type: EntityType,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api.kernel_get_system(),
            on_allocate_node_id(&mut api.system_module_api(), entity_type)
        )
    }

    #[trace_resources]
    pub fn on_create_node(
        api: &mut impl SystemBasedKernelInternalApi,
        event: &CreateNodeEvent,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api.kernel_get_system(),
            on_create_node(&mut api.system_module_api(), event)
        )
    }

    #[trace_resources]
    pub fn on_move_module(
        api: &mut impl SystemBasedKernelInternalApi,
        event: &MoveModuleEvent,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api.kernel_get_system(),
            on_move_module(&mut api.system_module_api(), event)
        )
    }

    #[trace_resources]
    pub fn on_drop_node(
        api: &mut impl SystemBasedKernelInternalApi,
        event: &DropNodeEvent,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api.kernel_get_system(),
            on_drop_node(&mut api.system_module_api(), event)
        )
    }

    #[trace_resources]
    pub fn on_mark_substate_as_transient(
        api: &mut impl SystemBasedKernelInternalApi,
        node_id: &NodeId,
        partition_number: &PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api.kernel_get_system(),
            on_mark_substate_as_transient(
                &mut api.system_module_api(),
                node_id,
                partition_number,
                substate_key
            )
        )
    }

    #[trace_resources]
    pub fn on_open_substate(
        api: &mut impl SystemBasedKernelInternalApi,
        event: &OpenSubstateEvent,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api.kernel_get_system(),
            on_open_substate(&mut api.system_module_api(), event)
        )
    }

    #[trace_resources(log=event.is_about_heap())]
    pub fn on_read_substate(
        api: &mut impl SystemBasedKernelInternalApi,
        event: &ReadSubstateEvent,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api.kernel_get_system(),
            on_read_substate(&mut api.system_module_api(), event)
        )
    }

    #[trace_resources]
    pub fn on_write_substate(
        api: &mut impl SystemBasedKernelInternalApi,
        event: &WriteSubstateEvent,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api.kernel_get_system(),
            on_write_substate(&mut api.system_module_api(), event)
        )
    }

    #[trace_resources]
    pub fn on_close_substate(
        api: &mut impl SystemBasedKernelInternalApi,
        event: &CloseSubstateEvent,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api.kernel_get_system(),
            on_close_substate(&mut api.system_module_api(), event)
        )
    }

    #[trace_resources]
    pub fn on_set_substate(
        api: &mut impl SystemBasedKernelInternalApi,
        event: &SetSubstateEvent,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api.kernel_get_system(),
            on_set_substate(&mut api.system_module_api(), event)
        )
    }

    #[trace_resources]
    pub fn on_remove_substate(
        api: &mut impl SystemBasedKernelInternalApi,
        event: &RemoveSubstateEvent,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api.kernel_get_system(),
            on_remove_substate(&mut api.system_module_api(), event)
        )
    }

    #[trace_resources]
    pub fn on_scan_keys(
        api: &mut impl SystemBasedKernelInternalApi,
        event: &ScanKeysEvent,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api.kernel_get_system(),
            on_scan_keys(&mut api.system_module_api(), event)
        )
    }

    #[trace_resources]
    pub fn on_drain_substates(
        api: &mut impl SystemBasedKernelInternalApi,
        event: &DrainSubstatesEvent,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api.kernel_get_system(),
            on_drain_substates(&mut api.system_module_api(), event)
        )
    }

    #[trace_resources]
    pub fn on_scan_sorted_substates(
        api: &mut impl SystemBasedKernelInternalApi,
        event: &ScanSortedSubstatesEvent,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api.kernel_get_system(),
            on_scan_sorted_substates(&mut api.system_module_api(), event)
        )
    }

    #[trace_resources]
    pub fn on_get_stack_id(
        api: &mut impl SystemBasedKernelInternalApi,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api.kernel_get_system(),
            on_get_stack_id(&mut api.system_module_api())
        )
    }

    #[trace_resources]
    pub fn on_switch_stack(
        api: &mut impl SystemBasedKernelInternalApi,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api.kernel_get_system(),
            on_switch_stack(&mut api.system_module_api())
        )
    }

    #[trace_resources]
    pub fn on_send_to_stack(
        api: &mut impl SystemBasedKernelInternalApi,
        data_len: usize,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api.kernel_get_system(),
            on_send_to_stack(&mut api.system_module_api(), data_len)
        )
    }

    #[trace_resources]
    pub fn on_set_call_frame_data(
        api: &mut impl SystemBasedKernelInternalApi,
        data_len: usize,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api.kernel_get_system(),
            on_set_call_frame_data(&mut api.system_module_api(), data_len)
        )
    }

    #[trace_resources]
    pub fn on_get_owned_nodes(
        api: &mut impl SystemBasedKernelInternalApi,
    ) -> Result<(), RuntimeError> {
        internal_call_dispatch!(
            api.kernel_get_system(),
            on_get_owned_nodes(&mut api.system_module_api())
        )
    }
}

impl SystemModuleMixer {
    // Note that module mixer is called by both kernel and system.
    // - Kernel uses the `SystemModule<SystemConfig<V>>` trait above;
    // - System uses methods defined below (TODO: add a trait?)

    pub fn on_call_method<Y: SystemBasedKernelApi>(
        api: &mut SystemService<Y>,
        receiver: &NodeId,
        module_id: ModuleId,
        direct_access: bool,
        ident: &str,
        args: &IndexedScryptoValue,
    ) -> Result<NodeId, RuntimeError> {
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

    pub fn on_call_method_finish<Y: SystemBasedKernelApi>(
        api: &mut SystemService<Y>,
        auth_zone: NodeId,
    ) -> Result<(), RuntimeError> {
        AuthModule::on_call_method_finish(api, auth_zone)
    }

    pub fn on_call_function<Y: SystemBasedKernelApi>(
        api: &mut SystemService<Y>,
        blueprint_id: &BlueprintId,
        ident: &str,
    ) -> Result<NodeId, RuntimeError> {
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

    pub fn on_call_function_finish<Y: SystemBasedKernelApi>(
        api: &mut SystemService<Y>,
        auth_zone: NodeId,
    ) -> Result<(), RuntimeError> {
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

    pub fn costing(&self) -> Option<&CostingModule> {
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

    pub fn costing_mut_even_if_disabled(&mut self) -> &mut CostingModule {
        &mut self.costing
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
