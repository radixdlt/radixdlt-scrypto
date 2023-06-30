use super::{BalanceChange, StateUpdateSummary};
use crate::blueprints::consensus_manager::EpochChangeEvent;
use crate::errors::*;
use crate::system::system_modules::costing::FeeSummary;
use crate::system::system_modules::execution_trace::{
    ExecutionTrace, ResourceChange, WorktopChange,
};
use crate::track::StateUpdates;
use crate::types::*;
use colored::*;
use radix_engine_interface::address::AddressDisplayContext;
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use radix_engine_interface::data::scrypto::ScryptoDecode;
use radix_engine_interface::types::*;
use sbor::representations::*;
use utils::ContextualDisplay;

#[derive(Debug, Clone, Default, ScryptoSbor)]
pub struct ResourcesUsage {
    pub heap_allocations_sum: usize,
    pub heap_peak_memory: usize,
    pub cpu_cycles: u64,
}

#[derive(Debug, Clone, ScryptoSbor, Default)]
pub struct TransactionExecutionTrace {
    pub execution_traces: Vec<ExecutionTrace>,
    pub resource_changes: IndexMap<usize, Vec<ResourceChange>>,
    pub fee_locks: FeeLocks,
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

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, Default)]
pub struct FeeLocks {
    pub lock: Decimal,
    pub contingent_lock: Decimal,
}

/// Captures whether a transaction should be committed, and its other results
#[derive(Debug, Clone, ScryptoSbor)]
pub enum TransactionResult {
    Commit(CommitResult),
    Reject(RejectResult),
    Abort(AbortResult),
}

impl TransactionResult {
    pub fn is_commit_success(&self) -> bool {
        match self {
            TransactionResult::Commit(c) => matches!(c.outcome, TransactionOutcome::Success(_)),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct CommitResult {
    pub state_updates: StateUpdates,
    pub state_update_summary: StateUpdateSummary,
    pub outcome: TransactionOutcome,
    pub fee_summary: FeeSummary,
    pub application_events: Vec<(EventTypeIdentifier, Vec<u8>)>,
    pub application_logs: Vec<(Level, String)>,
    /// Optional, only when `EnabledModule::ExecutionTrace` is ON.
    /// Mainly for transaction preview.
    pub execution_trace: TransactionExecutionTrace,
}

impl CommitResult {
    pub fn empty_with_outcome(outcome: TransactionOutcome) -> Self {
        Self {
            state_updates: Default::default(),
            state_update_summary: Default::default(),
            outcome,
            fee_summary: Default::default(),
            application_events: Default::default(),
            application_logs: Default::default(),
            execution_trace: Default::default(),
        }
    }

    pub fn next_epoch(&self) -> Option<EpochChangeEvent> {
        // Note: Node should use a well-known index id
        for (ref event_type_id, ref event_data) in self.application_events.iter() {
            if let EventTypeIdentifier(
                Emitter::Function(node_id, ObjectModuleId::Main, ..)
                | Emitter::Method(node_id, ObjectModuleId::Main),
                ..,
            ) = event_type_id
            {
                if node_id == CONSENSUS_MANAGER_PACKAGE.as_node_id()
                    || node_id.entity_type() == Some(EntityType::GlobalConsensusManager)
                {
                    if let Ok(epoch_change_event) = scrypto_decode::<EpochChangeEvent>(&event_data)
                    {
                        return Some(epoch_change_event);
                    }
                }
            }
        }
        None
    }

    pub fn new_package_addresses(&self) -> &Vec<PackageAddress> {
        &self.state_update_summary.new_packages
    }

    pub fn new_component_addresses(&self) -> &Vec<ComponentAddress> {
        &self.state_update_summary.new_components
    }

    pub fn new_resource_addresses(&self) -> &Vec<ResourceAddress> {
        &self.state_update_summary.new_resources
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

/// Captures whether a transaction's commit outcome is Success or Failure
#[derive(Debug, Clone, ScryptoSbor)]
pub enum TransactionOutcome {
    Success(Vec<InstructionOutput>),
    Failure(RuntimeError),
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

#[derive(Debug, Clone, ScryptoSbor)]
pub struct RejectResult {
    pub error: RejectionError,
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct AbortResult {
    pub reason: AbortReason,
}

#[derive(Debug, Clone, Display, PartialEq, Eq, Sbor)]
pub enum AbortReason {
    ConfiguredAbortTriggeredOnFeeLoanRepayment,
}

/// Represents a transaction receipt.
#[derive(Clone, ScryptoSbor)]
pub struct TransactionReceipt {
    pub transaction_result: TransactionResult,
    /// Optional, only when compile-time feature flag `resources_usage` is ON.
    pub resources_usage: ResourcesUsage,
}

impl TransactionReceipt {
    /// An empty receipt for merging changes into.
    pub fn empty_with_commit(commit_result: CommitResult) -> Self {
        Self {
            transaction_result: TransactionResult::Commit(commit_result),
            resources_usage: Default::default(),
        }
    }

    pub fn is_commit_success(&self) -> bool {
        matches!(
            self.transaction_result,
            TransactionResult::Commit(CommitResult {
                outcome: TransactionOutcome::Success(_),
                ..
            })
        )
    }

    pub fn is_commit_failure(&self) -> bool {
        matches!(
            self.transaction_result,
            TransactionResult::Commit(CommitResult {
                outcome: TransactionOutcome::Failure(_),
                ..
            })
        )
    }

    pub fn is_rejection(&self) -> bool {
        matches!(self.transaction_result, TransactionResult::Reject(_))
    }

    pub fn expect_commit(&self, success: bool) -> &CommitResult {
        match &self.transaction_result {
            TransactionResult::Commit(c) => {
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
            TransactionResult::Reject(_) => panic!("Transaction was rejected"),
            TransactionResult::Abort(_) => panic!("Transaction was aborted"),
        }
    }

    pub fn expect_commit_success(&self) -> &CommitResult {
        self.expect_commit(true)
    }

    pub fn expect_commit_failure(&self) -> &CommitResult {
        self.expect_commit(false)
    }

    pub fn expect_rejection(&self) -> &RejectionError {
        match &self.transaction_result {
            TransactionResult::Commit(..) => panic!("Expected rejection but was commit"),
            TransactionResult::Reject(ref r) => &r.error,
            TransactionResult::Abort(..) => panic!("Expected rejection but was abort"),
        }
    }

    pub fn expect_abortion(&self) -> &AbortReason {
        match &self.transaction_result {
            TransactionResult::Commit(..) => panic!("Expected abortion but was commit"),
            TransactionResult::Reject(..) => panic!("Expected abortion but was reject"),
            TransactionResult::Abort(ref r) => &r.reason,
        }
    }

    pub fn expect_not_success(&self) {
        match &self.transaction_result {
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
        F: Fn(&RejectionError) -> bool,
    {
        match &self.transaction_result {
            TransactionResult::Commit(..) => panic!("Expected rejection but was committed"),
            TransactionResult::Reject(result) => {
                if !f(&result.error) {
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
        match &self.transaction_result {
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
        let result = &self.transaction_result;
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
                TransactionResult::Reject(r) => format!("REJECTED: {}", r.error).red(),
                TransactionResult::Abort(a) => format!("ABORTED: {}", a.reason).bright_red(),
            },
        )?;

        if let TransactionResult::Commit(c) = &result {
            write!(
                f,
                "\n{} {} XRD used for execution, {} XRD used for royalty, {} XRD in bad debt",
                "Transaction Fee:".bold().green(),
                c.fee_summary.total_execution_cost_xrd,
                c.fee_summary.total_royalty_cost_xrd,
                c.fee_summary.total_bad_debt_xrd,
            )?;

            write!(
                f,
                "\n{} {} limit, {} consumed, {} XRD per cost unit, {}% tip",
                "Cost Units:".bold().green(),
                c.fee_summary.cost_unit_limit,
                c.fee_summary.execution_cost_sum,
                c.fee_summary.cost_unit_price,
                c.fee_summary.tip_percentage
            )?;

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
