use super::{BalanceChange, CostingParameters, StateUpdateSummary};
use crate::blueprints::consensus_manager::EpochChangeEvent;
use crate::errors::*;
use crate::system::system_modules::costing::{FeeReserveFinalizationSummary, RoyaltyRecipient};
use crate::system::system_modules::execution_trace::{
    ExecutionTrace, ResourceChange, WorktopChange,
};
use crate::track::StateUpdates;
use crate::transaction::SystemStructure;
use crate::types::*;
use colored::*;
use radix_engine_interface::address::AddressDisplayContext;
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use radix_engine_interface::data::scrypto::ScryptoDecode;
use radix_engine_interface::types::*;
use sbor::representations::*;
use transaction::prelude::TransactionCostingParameters;
use utils::ContextualDisplay;

#[derive(Clone, ScryptoSbor)]
pub struct TransactionReceipt {
    /// Costing parameters
    pub costing_parameters: CostingParameters,
    /// Transaction costing parameters
    pub transaction_costing_parameters: TransactionCostingParameters,
    /// Transaction costing summary
    pub costing_summary: TransactionCostingSummary,
    /// Transaction costing breakdown
    pub costing_details: TransactionCostingDetails,
    /// Transaction result
    pub result: TransactionResult,
    /// Optional, only when compile-time feature flag `resources_usage` is ON.
    pub resources_usage: ResourcesUsage,
}

#[derive(Default, Debug, Clone, ScryptoSbor)]
pub struct TransactionCostingSummary {
    /// Total execution cost units consumed.
    pub total_execution_cost_units_consumed: u32,
    /// Total finalization cost units consumed.
    pub total_finalization_cost_units_consumed: u32,

    /// Total execution cost in XRD.
    pub total_execution_cost_in_xrd: Decimal,
    /// Total finalization cost in XRD.
    pub total_finalization_cost_in_xrd: Decimal,
    /// Total tipping cost in XRD.
    pub total_tipping_cost_in_xrd: Decimal,
    /// Total storage cost in XRD.
    pub total_storage_cost_in_xrd: Decimal,
    /// Total royalty cost in XRD.
    pub total_royalty_cost_in_xrd: Decimal,
}

#[derive(Default, Debug, Clone, ScryptoSbor)]
pub struct TransactionCostingDetails {
    /// Execution cost breakdown
    ///
    /// Available only if `ExecutionConfig::enable_cost_breakdown` is true
    pub execution_cost_breakdown: BTreeMap<String, u32>,
    /// Finalization cost breakdown
    ///
    /// Available only if `ExecutionConfig::enable_cost_breakdown` is true
    pub finalization_cost_breakdown: BTreeMap<String, u32>,
}

/// Captures whether a transaction should be committed, and its other results
#[derive(Debug, Clone, ScryptoSbor)]
pub enum TransactionResult {
    Commit(CommitResult),
    Reject(RejectResult),
    Abort(AbortResult),
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct CommitResult {
    pub state_updates: StateUpdates,
    pub state_update_summary: StateUpdateSummary,
    pub fee_source: FeeSource,
    pub fee_destination: FeeDestination,
    pub outcome: TransactionOutcome,
    pub application_events: Vec<(EventTypeIdentifier, Vec<u8>)>,
    pub application_logs: Vec<(Level, String)>,
    pub system_structure: SystemStructure,
    /// Empty unless `EnabledModule::ExecutionTrace` is turned on.
    /// Mainly for transaction preview.
    pub execution_trace: TransactionExecutionTrace,
}

#[derive(Debug, Clone, Default, ScryptoSbor)]
pub struct FeeSource {
    pub paying_vaults: IndexMap<NodeId, Decimal>,
}

#[derive(Debug, Clone, Default, ScryptoSbor)]
pub struct FeeDestination {
    pub to_proposer: Decimal,
    pub to_validator_set: Decimal,
    pub to_burn: Decimal,
    pub to_royalty_recipients: IndexMap<RoyaltyRecipient, Decimal>,
}

/// Captures whether a transaction's commit outcome is Success or Failure
#[derive(Debug, Clone, ScryptoSbor)]
pub enum TransactionOutcome {
    Success(Vec<InstructionOutput>),
    Failure(RuntimeError),
}

#[derive(Debug, Clone, ScryptoSbor, Default)]
pub struct TransactionExecutionTrace {
    pub execution_traces: Vec<ExecutionTrace>,
    pub resource_changes: IndexMap<usize, Vec<ResourceChange>>,
    pub fee_locks: FeeLocks,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, Default)]
pub struct FeeLocks {
    pub lock: Decimal,
    pub contingent_lock: Decimal,
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct RejectResult {
    pub reason: RejectionReason,
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct AbortResult {
    pub reason: AbortReason,
}

#[derive(Debug, Clone, Display, PartialEq, Eq, Sbor)]
pub enum AbortReason {
    ConfiguredAbortTriggeredOnFeeLoanRepayment,
}

#[derive(Debug, Clone, Default, ScryptoSbor)]
pub struct ResourcesUsage {
    pub heap_allocations_sum: usize,
    pub heap_peak_memory: usize,
    pub cpu_cycles: u64,
}

impl TransactionExecutionTrace {
    pub fn worktop_changes(&self) -> IndexMap<usize, Vec<WorktopChange>> {
        let mut aggregator = index_map_new::<usize, Vec<WorktopChange>>();
        for trace in &self.execution_traces {
            trace.worktop_changes(&mut aggregator)
        }
        aggregator
    }
}

impl TransactionResult {
    pub fn is_commit_success(&self) -> bool {
        match self {
            TransactionResult::Commit(c) => matches!(c.outcome, TransactionOutcome::Success(_)),
            _ => false,
        }
    }
}

impl CommitResult {
    pub fn empty_with_outcome(outcome: TransactionOutcome) -> Self {
        Self {
            state_updates: Default::default(),
            state_update_summary: Default::default(),
            fee_source: Default::default(),
            fee_destination: Default::default(),
            outcome,
            application_events: Default::default(),
            application_logs: Default::default(),
            system_structure: Default::default(),
            execution_trace: Default::default(),
        }
    }

    pub fn next_epoch(&self) -> Option<EpochChangeEvent> {
        // Note: Node should use a well-known index id
        for (ref event_type_id, ref event_data) in self.application_events.iter() {
            let is_consensus_manager = match &event_type_id.0 {
                Emitter::Method(node_id, ObjectModuleId::Main)
                    if node_id.entity_type() == Some(EntityType::GlobalConsensusManager) =>
                {
                    true
                }
                Emitter::Function(blueprint_id)
                    if blueprint_id.package_address.eq(&CONSENSUS_MANAGER_PACKAGE) =>
                {
                    true
                }
                _ => false,
            };

            if is_consensus_manager {
                if let Ok(epoch_change_event) = scrypto_decode::<EpochChangeEvent>(&event_data) {
                    return Some(epoch_change_event);
                }
            }
        }
        None
    }

    pub fn new_package_addresses(&self) -> &IndexSet<PackageAddress> {
        &self.state_update_summary.new_packages
    }

    pub fn new_component_addresses(&self) -> &IndexSet<ComponentAddress> {
        &self.state_update_summary.new_components
    }

    pub fn new_resource_addresses(&self) -> &IndexSet<ResourceAddress> {
        &self.state_update_summary.new_resources
    }

    pub fn new_vault_addresses(&self) -> &IndexSet<InternalAddress> {
        &self.state_update_summary.new_vaults
    }

    pub fn balance_changes(
        &self,
    ) -> &IndexMap<GlobalAddress, IndexMap<ResourceAddress, BalanceChange>> {
        &self.state_update_summary.balance_changes
    }

    pub fn direct_vault_updates(
        &self,
    ) -> &IndexMap<NodeId, IndexMap<ResourceAddress, BalanceChange>> {
        &self.state_update_summary.direct_vault_updates
    }

    pub fn output<T: ScryptoDecode>(&self, nth: usize) -> T {
        match &self.outcome {
            TransactionOutcome::Success(o) => match o.get(nth) {
                Some(InstructionOutput::CallReturn(value)) => {
                    scrypto_decode::<T>(value).expect("Output can't be converted")
                }
                _ => panic!("No output for [{}]", nth),
            },
            TransactionOutcome::Failure(_) => panic!("Transaction failed"),
        }
    }
}

impl TransactionOutcome {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    pub fn expect_success(&self) -> &Vec<InstructionOutput> {
        match self {
            TransactionOutcome::Success(results) => results,
            TransactionOutcome::Failure(error) => panic!("Outcome was a failure: {}", error),
        }
    }

    pub fn expect_failure(&self) -> &RuntimeError {
        match self {
            TransactionOutcome::Success(_) => panic!("Outcome was an unexpected success"),
            TransactionOutcome::Failure(error) => error,
        }
    }

    pub fn success_or_else<E, F: Fn(&RuntimeError) -> E>(
        &self,
        f: F,
    ) -> Result<&Vec<InstructionOutput>, E> {
        match self {
            TransactionOutcome::Success(results) => Ok(results),
            TransactionOutcome::Failure(error) => Err(f(error)),
        }
    }
}

impl TransactionReceipt {
    /// An empty receipt for merging changes into.
    pub fn empty_with_commit(commit_result: CommitResult) -> Self {
        Self {
            costing_parameters: Default::default(),
            transaction_costing_parameters: Default::default(),
            costing_summary: Default::default(),
            costing_details: Default::default(),
            result: TransactionResult::Commit(commit_result),
            resources_usage: Default::default(),
        }
    }

    pub fn is_commit_success(&self) -> bool {
        matches!(
            self.result,
            TransactionResult::Commit(CommitResult {
                outcome: TransactionOutcome::Success(_),
                ..
            })
        )
    }

    pub fn is_commit_failure(&self) -> bool {
        matches!(
            self.result,
            TransactionResult::Commit(CommitResult {
                outcome: TransactionOutcome::Failure(_),
                ..
            })
        )
    }

    pub fn is_rejection(&self) -> bool {
        matches!(self.result, TransactionResult::Reject(_))
    }

    pub fn expect_commit_ignore_outcome(&self) -> &CommitResult {
        match &self.result {
            TransactionResult::Commit(c) => c,
            TransactionResult::Reject(_) => panic!("Transaction was rejected"),
            TransactionResult::Abort(_) => panic!("Transaction was aborted"),
        }
    }

    pub fn expect_commit(&self, success: bool) -> &CommitResult {
        let c = self.expect_commit_ignore_outcome();
        if c.outcome.is_success() != success {
            panic!(
                "Expected {} but was {}: {:?}",
                if success { "success" } else { "failure" },
                if c.outcome.is_success() {
                    "success"
                } else {
                    "failure"
                },
                c.outcome
            )
        }
        c
    }

    pub fn expect_commit_success(&self) -> &CommitResult {
        self.expect_commit(true)
    }

    pub fn expect_commit_failure(&self) -> &CommitResult {
        self.expect_commit(false)
    }

    pub fn expect_rejection(&self) -> &RejectionReason {
        match &self.result {
            TransactionResult::Commit(..) => panic!("Expected rejection but was commit"),
            TransactionResult::Reject(ref r) => &r.reason,
            TransactionResult::Abort(..) => panic!("Expected rejection but was abort"),
        }
    }

    pub fn expect_abortion(&self) -> &AbortReason {
        match &self.result {
            TransactionResult::Commit(..) => panic!("Expected abortion but was commit"),
            TransactionResult::Reject(..) => panic!("Expected abortion but was reject"),
            TransactionResult::Abort(ref r) => &r.reason,
        }
    }

    pub fn expect_not_success(&self) {
        match &self.result {
            TransactionResult::Commit(c) => {
                if c.outcome.is_success() {
                    panic!("Transaction succeeded unexpectedly")
                }
            }
            TransactionResult::Reject(..) => {}
            TransactionResult::Abort(..) => {}
        }
    }

    pub fn expect_specific_rejection<F>(&self, f: F)
    where
        F: Fn(&RejectionReason) -> bool,
    {
        match &self.result {
            TransactionResult::Commit(..) => panic!("Expected rejection but was committed"),
            TransactionResult::Reject(result) => {
                if !f(&result.reason) {
                    panic!(
                        "Expected specific rejection but was different error:\n{:?}",
                        self
                    );
                }
            }
            TransactionResult::Abort(..) => panic!("Expected rejection but was abort"),
        }
    }

    pub fn expect_failure(&self) -> &RuntimeError {
        match &self.result {
            TransactionResult::Commit(c) => match &c.outcome {
                TransactionOutcome::Success(_) => panic!("Expected failure but was success"),
                TransactionOutcome::Failure(error) => error,
            },
            TransactionResult::Reject(_) => panic!("Transaction was rejected"),
            TransactionResult::Abort(..) => panic!("Transaction was aborted"),
        }
    }

    pub fn expect_specific_failure<F>(&self, f: F)
    where
        F: Fn(&RuntimeError) -> bool,
    {
        if !f(self.expect_failure()) {
            panic!(
                "Expected specific failure but was different error:\n{:?}",
                self
            );
        }
    }

    pub fn expect_auth_failure(&self) {
        self.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::SystemModuleError(SystemModuleError::AuthError(..))
            )
        })
    }

    pub fn expect_auth_assertion_failure(&self) {
        self.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::SystemError(SystemError::AssertAccessRuleFailed)
            )
        })
    }

    pub fn expect_auth_mutability_failure(&self) {
        self.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::SystemError(SystemError::MutatingImmutableSubstate)
            )
        })
    }
}

macro_rules! prefix {
    ($i:expr, $list:expr) => {
        if $i == $list.len() - 1 {
            "└─"
        } else {
            "├─"
        }
    };
}

impl fmt::Debug for TransactionReceipt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.display(TransactionReceiptDisplayContext::default())
        )
    }
}

#[derive(Default)]
pub struct TransactionReceiptDisplayContext<'a> {
    pub encoder: Option<&'a AddressBech32Encoder>,
    pub schema_lookup_callback:
        Option<Box<dyn Fn(&EventTypeIdentifier) -> Option<(LocalTypeIndex, ScryptoSchema)> + 'a>>,
}

impl<'a> TransactionReceiptDisplayContext<'a> {
    pub fn display_context(&self) -> ScryptoValueDisplayContext<'a> {
        ScryptoValueDisplayContext::with_optional_bech32(self.encoder)
    }

    pub fn address_display_context(&self) -> AddressDisplayContext<'a> {
        AddressDisplayContext {
            encoder: self.encoder,
        }
    }

    pub fn lookup_schema(
        &self,
        event_type_identifier: &EventTypeIdentifier,
    ) -> Option<(LocalTypeIndex, ScryptoSchema)> {
        match self.schema_lookup_callback {
            Some(ref callback) => {
                let callback = callback.as_ref();
                callback(event_type_identifier)
            }
            None => None,
        }
    }
}

impl<'a> From<&'a AddressBech32Encoder> for TransactionReceiptDisplayContext<'a> {
    fn from(encoder: &'a AddressBech32Encoder) -> Self {
        Self {
            encoder: Some(encoder),
            schema_lookup_callback: None,
        }
    }
}

impl<'a> From<Option<&'a AddressBech32Encoder>> for TransactionReceiptDisplayContext<'a> {
    fn from(encoder: Option<&'a AddressBech32Encoder>) -> Self {
        Self {
            encoder,
            schema_lookup_callback: None,
        }
    }
}

pub struct TransactionReceiptDisplayContextBuilder<'a>(TransactionReceiptDisplayContext<'a>);

impl<'a> TransactionReceiptDisplayContextBuilder<'a> {
    pub fn new() -> Self {
        Self(TransactionReceiptDisplayContext {
            encoder: None,
            schema_lookup_callback: None,
        })
    }

    pub fn encoder(mut self, encoder: &'a AddressBech32Encoder) -> Self {
        self.0.encoder = Some(encoder);
        self
    }

    pub fn schema_lookup_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&EventTypeIdentifier) -> Option<(LocalTypeIndex, ScryptoSchema)> + 'a,
    {
        self.0.schema_lookup_callback = Some(Box::new(callback));
        self
    }

    pub fn build(self) -> TransactionReceiptDisplayContext<'a> {
        self.0
    }
}

impl<'a> ContextualDisplay<TransactionReceiptDisplayContext<'a>> for TransactionReceipt {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &TransactionReceiptDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        let result = &self.result;
        let scrypto_value_display_context = context.display_context();
        let address_display_context = context.address_display_context();

        write!(
            f,
            "{} {}",
            "Transaction Status:".bold().green(),
            match result {
                TransactionResult::Commit(c) => match &c.outcome {
                    TransactionOutcome::Success(_) => "COMMITTED SUCCESS".green(),
                    TransactionOutcome::Failure(e) => format!("COMMITTED FAILURE: {}", e).red(),
                },
                TransactionResult::Reject(r) => format!("REJECTED: {}", r.reason).red(),
                TransactionResult::Abort(a) => format!("ABORTED: {}", a.reason).bright_red(),
            },
        )?;

        write!(
                f,
                "\n{} Execution => {} XRD, Finalization => {} XRD, Tipping => {} XRD, Storage => {} XRD, Royalty => {} XRD",
                "Transaction Cost:".bold().green(),
                self.costing_summary.total_execution_cost_in_xrd,
                self.costing_summary.total_finalization_cost_in_xrd,
                self.costing_summary.total_tipping_cost_in_xrd,
                self.costing_summary.total_storage_cost_in_xrd,
                self.costing_summary.total_royalty_cost_in_xrd,
            )?;

        write!(
            f,
            "\n{} {} limit, {} consumed, {} XRD per unit",
            "Execution Cost Units:".bold().green(),
            self.costing_parameters.execution_cost_unit_limit,
            self.costing_summary.total_execution_cost_units_consumed,
            self.costing_parameters.execution_cost_unit_price,
        )?;

        write!(
            f,
            "\n{} {} limit, {} consumed, {} XRD per unit",
            "Finalization Cost Units:".bold().green(),
            self.costing_parameters.finalization_cost_unit_limit,
            self.costing_summary.total_finalization_cost_units_consumed,
            self.costing_parameters.finalization_cost_unit_price,
        )?;

        if let TransactionResult::Commit(c) = &result {
            write!(
                f,
                "\n{} {}",
                "Logs:".bold().green(),
                c.application_logs.len()
            )?;
            for (i, (level, msg)) in c.application_logs.iter().enumerate() {
                let (l, m) = match level {
                    Level::Error => ("ERROR".red(), msg.red()),
                    Level::Warn => ("WARN".yellow(), msg.yellow()),
                    Level::Info => ("INFO".green(), msg.green()),
                    Level::Debug => ("DEBUG".cyan(), msg.cyan()),
                    Level::Trace => ("TRACE".normal(), msg.normal()),
                };
                write!(f, "\n{} [{:5}] {}", prefix!(i, c.application_logs), l, m)?;
            }

            write!(
                f,
                "\n{} {}",
                "Events:".bold().green(),
                c.application_events.len()
            )?;
            for (i, (event_type_identifier, event_data)) in c.application_events.iter().enumerate()
            {
                if context.schema_lookup_callback.is_some() {
                    display_event_with_network_and_schema_context(
                        f,
                        prefix!(i, c.application_events),
                        event_type_identifier,
                        event_data,
                        context,
                    )?;
                } else {
                    display_event_with_network_context(
                        f,
                        prefix!(i, c.application_events),
                        event_type_identifier,
                        event_data,
                        context,
                    )?;
                }
            }

            if let TransactionOutcome::Success(outputs) = &c.outcome {
                write!(f, "\n{} {}", "Outputs:".bold().green(), outputs.len())?;
                for (i, output) in outputs.iter().enumerate() {
                    write!(
                        f,
                        "\n{} {}",
                        prefix!(i, outputs),
                        match output {
                            InstructionOutput::CallReturn(x) => IndexedScryptoValue::from_slice(&x)
                                .expect("Impossible case! Instruction output can't be decoded")
                                .to_string(ValueDisplayParameters::Schemaless {
                                    display_mode: DisplayMode::RustLike,
                                    print_mode: PrintMode::MultiLine {
                                        indent_size: 2,
                                        base_indent: 3,
                                        first_line_indent: 0
                                    },
                                    custom_context: scrypto_value_display_context
                                }),
                            InstructionOutput::None => "None".to_string(),
                        }
                    )?;
                }
            }

            let mut balance_changes = Vec::new();
            for (address, map) in c.balance_changes() {
                for (resource, delta) in map {
                    balance_changes.push((address, resource, delta));
                }
            }
            write!(
                f,
                "\n{} {}",
                "Balance Changes:".bold().green(),
                balance_changes.len()
            )?;
            for (i, (address, resource, delta)) in balance_changes.iter().enumerate() {
                write!(
                    f,
                    // NB - we use ResAddr instead of Resource to protect people who read new resources as
                    //      `Resource: ` from the receipts (see eg resim.sh)
                    "\n{} Entity: {}\n   ResAddr: {}\n   Change: {}",
                    prefix!(i, balance_changes),
                    address.display(address_display_context),
                    resource.display(address_display_context),
                    match delta {
                        BalanceChange::Fungible(d) => format!("{}", d),
                        BalanceChange::NonFungible { added, removed } => {
                            format!("+{:?}, -{:?}", added, removed)
                        }
                    }
                )?;
            }

            let mut direct_vault_updates = Vec::new();
            for (object_id, map) in c.direct_vault_updates() {
                for (resource, delta) in map {
                    direct_vault_updates.push((object_id, resource, delta));
                }
            }
            write!(
                f,
                "\n{} {}",
                "Direct Vault Updates:".bold().green(),
                direct_vault_updates.len()
            )?;
            for (i, (object_id, resource, delta)) in direct_vault_updates.iter().enumerate() {
                write!(
                    f,
                    // NB - we use ResAddr instead of Resource to protect people who read new resources as
                    //      `Resource: ` from the receipts (see eg resim.sh)
                    "\n{} Vault: {}\n   ResAddr: {}\n   Change: {}",
                    prefix!(i, direct_vault_updates),
                    hex::encode(object_id),
                    resource.display(address_display_context),
                    match delta {
                        BalanceChange::Fungible(d) => format!("{}", d),
                        BalanceChange::NonFungible { added, removed } => {
                            format!("+{:?}, -{:?}", added, removed)
                        }
                    }
                )?;
            }

            write!(
                f,
                "\n{} {}",
                "New Entities:".bold().green(),
                c.new_package_addresses().len()
                    + c.new_component_addresses().len()
                    + c.new_resource_addresses().len()
            )?;
            for (i, package_address) in c.new_package_addresses().iter().enumerate() {
                write!(
                    f,
                    "\n{} Package: {}",
                    prefix!(i, c.new_package_addresses()),
                    package_address.display(address_display_context)
                )?;
            }
            for (i, component_address) in c.new_component_addresses().iter().enumerate() {
                write!(
                    f,
                    "\n{} Component: {}",
                    prefix!(i, c.new_component_addresses()),
                    component_address.display(address_display_context)
                )?;
            }
            for (i, resource_address) in c.new_resource_addresses().iter().enumerate() {
                write!(
                    f,
                    "\n{} Resource: {}",
                    prefix!(i, c.new_resource_addresses()),
                    resource_address.display(address_display_context)
                )?;
            }
        }

        Ok(())
    }
}

fn display_event_with_network_context<'a, F: fmt::Write>(
    f: &mut F,
    prefix: &str,
    event_type_identifier: &EventTypeIdentifier,
    event_data: &Vec<u8>,
    receipt_context: &TransactionReceiptDisplayContext<'a>,
) -> Result<(), fmt::Error> {
    let event_data_value =
        IndexedScryptoValue::from_slice(&event_data).expect("Event must be decodable!");
    write!(
        f,
        "\n{} Emitter: {}\n   Local Type Index: {:?}\n   Data: {}",
        prefix,
        event_type_identifier
            .0
            .display(receipt_context.address_display_context()),
        event_type_identifier.1,
        event_data_value.display(ValueDisplayParameters::Schemaless {
            display_mode: DisplayMode::RustLike,
            print_mode: PrintMode::MultiLine {
                indent_size: 2,
                base_indent: 3,
                first_line_indent: 0
            },
            custom_context: receipt_context.display_context(),
        })
    )?;
    Ok(())
}

fn display_event_with_network_and_schema_context<'a, F: fmt::Write>(
    f: &mut F,
    prefix: &str,
    event_type_identifier: &EventTypeIdentifier,
    event_data: &Vec<u8>,
    receipt_context: &TransactionReceiptDisplayContext<'a>,
) -> Result<(), fmt::Error> {
    // Given the event type identifier, get the local type index and schema associated with it.
    let (local_type_index, schema) = receipt_context
        .lookup_schema(event_type_identifier)
        .map_or(Err(fmt::Error), Ok)?;

    // Based on the event data and schema, get an invertible json string representation.
    let event = ScryptoRawPayload::new_from_valid_slice(event_data).to_string(
        ValueDisplayParameters::Annotated {
            display_mode: DisplayMode::RustLike,
            print_mode: PrintMode::MultiLine {
                indent_size: 2,
                base_indent: 3,
                first_line_indent: 0,
            },
            custom_context: receipt_context.display_context(),
            schema: &schema,
            type_index: local_type_index,
        },
    );

    // Print the event information
    write!(
        f,
        "\n{} Emitter: {}\n   Event: {}",
        prefix,
        event_type_identifier
            .0
            .display(receipt_context.address_display_context()),
        event
    )?;
    Ok(())
}

impl From<FeeReserveFinalizationSummary> for TransactionCostingSummary {
    fn from(value: FeeReserveFinalizationSummary) -> Self {
        Self {
            total_execution_cost_units_consumed: value.total_execution_cost_units_consumed,
            total_finalization_cost_units_consumed: value.total_finalization_cost_units_consumed,
            total_execution_cost_in_xrd: value.total_execution_cost_in_xrd,
            total_finalization_cost_in_xrd: value.total_finalization_cost_in_xrd,
            total_tipping_cost_in_xrd: value.total_tipping_cost_in_xrd,
            total_storage_cost_in_xrd: value.total_storage_cost_in_xrd,
            total_royalty_cost_in_xrd: value.total_royalty_cost_in_xrd,
        }
    }
}

impl TransactionCostingSummary {
    pub fn total_cost(&self) -> Decimal {
        self.total_execution_cost_in_xrd
            + self.total_finalization_cost_in_xrd
            + self.total_tipping_cost_in_xrd
            + self.total_storage_cost_in_xrd
            + self.total_royalty_cost_in_xrd
    }

    pub fn network_fees(&self) -> Decimal {
        self.total_execution_cost_in_xrd
            + self.total_finalization_cost_in_xrd
            + self.total_storage_cost_in_xrd
    }

    //===================
    // For testing only
    //===================

    pub fn expected_reward_if_single_validator(&self) -> Decimal {
        self.expected_reward_as_proposer_if_single_validator()
            + self.expected_reward_as_active_validator_if_single_validator()
    }

    pub fn expected_reward_as_proposer_if_single_validator(&self) -> Decimal {
        self.total_tipping_cost_in_xrd * (TIPS_PROPOSER_SHARE_PERCENTAGE) / dec!(100)
            + (self.total_execution_cost_in_xrd
                + self.total_finalization_cost_in_xrd
                + self.total_storage_cost_in_xrd)
                * (NETWORK_FEES_PROPOSER_SHARE_PERCENTAGE)
                / dec!(100)
    }

    pub fn expected_reward_as_active_validator_if_single_validator(&self) -> Decimal {
        self.total_tipping_cost_in_xrd * (TIPS_VALIDATOR_SET_SHARE_PERCENTAGE) / dec!(100)
            + (self.total_execution_cost_in_xrd
                + self.total_finalization_cost_in_xrd
                + self.total_storage_cost_in_xrd)
                * (NETWORK_FEES_VALIDATOR_SET_SHARE_PERCENTAGE)
                / dec!(100)
    }
}
