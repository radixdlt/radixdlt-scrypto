use crate::errors::*;
use crate::internal_prelude::*;
use crate::kernel::kernel::KernelInit;
use crate::system::system_callback::*;
use crate::transaction::*;
use crate::vm::*;
use radix_common::constants::*;
use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use radix_substate_store_interface::interface::*;
use radix_transactions::model::*;

/// Protocol-defined costing parameters
#[derive(Debug, Copy, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct CostingParameters {
    /// The price of execution cost unit in XRD.
    pub execution_cost_unit_price: Decimal,
    /// The max number execution cost units to consume.
    pub execution_cost_unit_limit: u32,
    /// The number of execution cost units loaned from system.
    pub execution_cost_unit_loan: u32,

    /// The price of finalization cost unit in XRD.
    pub finalization_cost_unit_price: Decimal,
    /// The max number finalization cost units to consume.
    pub finalization_cost_unit_limit: u32,

    /// The price of USD in xrd
    pub usd_price: Decimal,
    /// The price of state storage in xrd
    pub state_storage_price: Decimal,
    /// The price of archive storage in xrd
    pub archive_storage_price: Decimal,
}

impl CostingParameters {
    pub fn latest() -> Self {
        Self::babylon_genesis()
    }

    #[cfg(not(feature = "coverage"))]
    pub fn babylon_genesis() -> Self {
        Self {
            execution_cost_unit_price: EXECUTION_COST_UNIT_PRICE_IN_XRD.try_into().unwrap(),
            execution_cost_unit_limit: EXECUTION_COST_UNIT_LIMIT,
            execution_cost_unit_loan: EXECUTION_COST_UNIT_LOAN,
            finalization_cost_unit_price: FINALIZATION_COST_UNIT_PRICE_IN_XRD.try_into().unwrap(),
            finalization_cost_unit_limit: FINALIZATION_COST_UNIT_LIMIT,
            usd_price: USD_PRICE_IN_XRD.try_into().unwrap(),
            state_storage_price: STATE_STORAGE_PRICE_IN_XRD.try_into().unwrap(),
            archive_storage_price: ARCHIVE_STORAGE_PRICE_IN_XRD.try_into().unwrap(),
        }
    }
    #[cfg(feature = "coverage")]
    pub fn babylon_genesis() -> Self {
        Self {
            execution_cost_unit_price: Decimal::zero(),
            execution_cost_unit_limit: u32::MAX,
            execution_cost_unit_loan: u32::MAX,
            finalization_cost_unit_price: Decimal::zero(),
            finalization_cost_unit_limit: u32::MAX,
            usd_price: USD_PRICE_IN_XRD.try_into().unwrap(),
            state_storage_price: Decimal::zero(),
            archive_storage_price: Decimal::zero(),
        }
    }

    pub fn with_execution_cost_unit_loan(mut self, loan_units: u32) -> Self {
        self.execution_cost_unit_loan = loan_units;
        self
    }

    pub fn with_execution_cost_unit_limit(mut self, limit: u32) -> Self {
        self.execution_cost_unit_limit = limit;
        self
    }

    pub fn with_finalization_cost_unit_limit(mut self, limit: u32) -> Self {
        self.finalization_cost_unit_limit = limit;
        self
    }
}

#[derive(Debug, Copy, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct LimitParameters {
    pub max_call_depth: usize,
    pub max_heap_substate_total_bytes: usize,
    pub max_track_substate_total_bytes: usize,
    pub max_substate_key_size: usize,
    pub max_substate_value_size: usize,
    pub max_invoke_input_size: usize,
    pub max_event_size: usize,
    pub max_log_size: usize,
    pub max_panic_message_size: usize,
    pub max_number_of_logs: usize,
    pub max_number_of_events: usize,
}

impl LimitParameters {
    pub fn babylon_genesis() -> Self {
        Self {
            max_call_depth: MAX_CALL_DEPTH,
            max_heap_substate_total_bytes: MAX_HEAP_SUBSTATE_TOTAL_BYTES,
            max_track_substate_total_bytes: MAX_TRACK_SUBSTATE_TOTAL_BYTES,
            max_substate_key_size: MAX_SUBSTATE_KEY_SIZE,
            max_substate_value_size: MAX_SUBSTATE_VALUE_SIZE,
            max_invoke_input_size: MAX_INVOKE_PAYLOAD_SIZE,
            max_event_size: MAX_EVENT_SIZE,
            max_log_size: MAX_LOG_SIZE,
            max_panic_message_size: MAX_PANIC_MESSAGE_SIZE,
            max_number_of_logs: MAX_NUMBER_OF_LOGS,
            max_number_of_events: MAX_NUMBER_OF_EVENTS,
        }
    }

    pub fn for_genesis_transaction() -> Self {
        Self {
            max_heap_substate_total_bytes: 512 * 1024 * 1024,
            max_track_substate_total_bytes: 512 * 1024 * 1024,
            max_number_of_events: 1024 * 1024,
            ..Self::babylon_genesis()
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemOverrides {
    pub disable_costing: bool,
    pub disable_limits: bool,
    pub disable_auth: bool,
    /// Whether to abort the transaction run when the loan is repaid.
    /// This is used when test-executing pending transactions.
    pub abort_when_loan_repaid: bool,
    /// This is required for pre-bottlenose testnets which need to override
    /// the default Mainnet network definition
    pub network_definition: Option<NetworkDefinition>,
    pub costing_parameters: Option<CostingParameters>,
    pub limit_parameters: Option<LimitParameters>,
}

impl SystemOverrides {
    const fn internal_default(network_definition: Option<NetworkDefinition>) -> Self {
        Self {
            disable_costing: false,
            disable_limits: false,
            disable_auth: false,
            abort_when_loan_repaid: false,
            network_definition,
            costing_parameters: None,
            limit_parameters: None,
        }
    }

    pub const fn default_with_no_network() -> Self {
        Self::internal_default(None)
    }

    pub const fn with_network(network_definition: NetworkDefinition) -> Self {
        Self::internal_default(Some(network_definition))
    }

    pub const fn set_costing_parameters(
        mut self,
        costing_parameters: Option<CostingParameters>,
    ) -> Self {
        self.costing_parameters = costing_parameters;
        self
    }

    pub fn set_abort_when_loan_repaid(mut self) -> Self {
        self.abort_when_loan_repaid = true;
        self
    }
}

impl Default for SystemOverrides {
    fn default() -> Self {
        Self::default_with_no_network()
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    // These parameters do not affect state execution but only affect side effects
    pub enable_kernel_trace: bool,
    pub enable_cost_breakdown: bool,
    pub execution_trace: Option<usize>,
    pub enable_debug_information: bool,

    pub system_overrides: Option<SystemOverrides>,
}

impl ExecutionConfig {
    /// Creates an `ExecutionConfig` using default configurations.
    /// This is internal. Clients should use `for_xxx` constructors instead.
    const fn default_with_no_network() -> Self {
        Self {
            enable_kernel_trace: false,
            enable_cost_breakdown: false,
            execution_trace: None,
            system_overrides: None,
            enable_debug_information: false,
        }
    }

    /// Creates an `ExecutionConfig` using default configurations.
    /// This is internal. Clients should use `for_xxx` constructors instead.
    fn default_with_network(network_definition: NetworkDefinition) -> Self {
        Self {
            system_overrides: Some(SystemOverrides::with_network(network_definition)),
            ..Self::default_with_no_network()
        }
    }

    pub fn for_genesis_transaction(network_definition: NetworkDefinition) -> Self {
        Self::for_auth_disabled_system_transaction(network_definition)
    }

    pub fn for_auth_disabled_system_transaction(network_definition: NetworkDefinition) -> Self {
        Self {
            system_overrides: Some(SystemOverrides {
                disable_costing: true,
                disable_limits: true,
                disable_auth: true,
                network_definition: Some(network_definition),
                ..Default::default()
            }),
            ..Self::default_with_no_network()
        }
    }

    pub fn for_system_transaction(network_definition: NetworkDefinition) -> Self {
        Self {
            system_overrides: Some(SystemOverrides {
                disable_costing: true,
                disable_limits: true,
                network_definition: Some(network_definition),
                ..Default::default()
            }),
            ..Self::default_with_no_network()
        }
    }

    pub fn for_validator_transaction(network_definition: NetworkDefinition) -> Self {
        Self {
            ..Self::default_with_network(network_definition)
        }
    }

    pub fn for_notarized_transaction(network_definition: NetworkDefinition) -> Self {
        Self {
            ..Self::default_with_network(network_definition)
        }
    }

    pub fn for_notarized_transaction_rejection_check(
        network_definition: NetworkDefinition,
    ) -> Self {
        Self::default_with_network(network_definition)
            .update_system_overrides(|overrides| overrides.set_abort_when_loan_repaid())
    }

    pub fn update_system_overrides(
        mut self,
        update: impl FnOnce(SystemOverrides) -> SystemOverrides,
    ) -> Self {
        self.system_overrides = Some(update(self.system_overrides.unwrap_or_default()));
        self
    }

    pub fn for_test_transaction() -> Self {
        Self {
            enable_cost_breakdown: true,
            ..Self::default_with_network(NetworkDefinition::simulator())
        }
    }

    pub fn for_debug_transaction() -> Self {
        Self {
            enable_debug_information: true,
            ..Self::for_test_transaction()
        }
    }

    pub fn for_preview(network_definition: NetworkDefinition) -> Self {
        Self {
            enable_cost_breakdown: true,
            execution_trace: Some(MAX_EXECUTION_TRACE_DEPTH),
            ..Self::default_with_network(network_definition)
        }
    }

    pub fn for_preview_no_auth(network_definition: NetworkDefinition) -> Self {
        Self {
            enable_cost_breakdown: true,
            execution_trace: Some(MAX_EXECUTION_TRACE_DEPTH),
            system_overrides: Some(SystemOverrides {
                disable_auth: true,
                network_definition: Some(network_definition),
                ..Default::default()
            }),
            ..Self::default_with_no_network()
        }
    }

    pub fn with_kernel_trace(mut self, enabled: bool) -> Self {
        self.enable_kernel_trace = enabled;
        self
    }

    pub fn with_execution_trace(mut self, depth: Option<usize>) -> Self {
        self.execution_trace = depth;
        self
    }

    pub fn with_cost_breakdown(mut self, enabled: bool) -> Self {
        self.enable_cost_breakdown = enabled;
        self
    }
}

pub fn execute_transaction<'v, V: VmInitialize>(
    substate_db: &impl SubstateDatabase,
    vm_modules: &'v V,
    execution_config: &ExecutionConfig,
    executable: impl AsRef<ExecutableTransaction>,
) -> TransactionReceipt {
    let vm_init = VmInit::load(substate_db, vm_modules);
    let system_init = SystemInit::load(substate_db, execution_config.clone(), vm_init);
    KernelInit::load(substate_db, system_init).execute(executable.as_ref())
}

pub fn execute_and_commit_transaction<'s, V: VmInitialize>(
    substate_db: &mut (impl SubstateDatabase + CommittableSubstateDatabase),
    vm_modules: &'s V,
    execution_config: &ExecutionConfig,
    executable: impl AsRef<ExecutableTransaction>,
) -> TransactionReceipt {
    let receipt = execute_transaction(substate_db, vm_modules, execution_config, executable);
    if let TransactionResult::Commit(commit) = &receipt.result {
        substate_db.commit(&commit.state_updates.create_database_updates());
    }
    receipt
}

pub enum TransactionResultType {
    Commit(Result<Vec<InstructionOutput>, RuntimeError>),
    Reject(RejectionReason),
    Abort(AbortReason),
}
