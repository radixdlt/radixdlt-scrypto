use super::{BalanceChange, StateUpdateSummary};
use crate::blueprints::epoch_manager::{EpochChangeEvent, Validator};
use crate::errors::*;
use crate::system::kernel_modules::costing::FeeSummary;
use crate::system::kernel_modules::execution_trace::{
    ExecutionTrace, ResourceChange, WorktopChange,
};
use crate::types::*;
use colored::*;
use radix_engine_interface::address::AddressDisplayContext;
use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use radix_engine_interface::data::scrypto::ScryptoDecode;
use radix_engine_interface::types::*;
use radix_engine_stores::interface::{StateDependencies, StateUpdates};
use utils::ContextualDisplay;

#[cfg(feature = "serde")]
use sbor::serde_serialization::{SborPayloadWithSchema, SerializationContext, SerializationMode};
#[cfg(feature = "serde")]
use utils::ContextualSerialize;

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
    pub state_dependencies: StateDependencies,
    pub state_update_summary: StateUpdateSummary,
    pub outcome: TransactionOutcome,
    pub fee_summary: FeeSummary,
    pub fee_payments: IndexMap<NodeId, Decimal>,
    pub application_events: Vec<(EventTypeIdentifier, Vec<u8>)>,
    pub application_logs: Vec<(Level, String)>,
}

impl CommitResult {
    pub fn next_epoch(&self) -> Option<(BTreeMap<ComponentAddress, Validator>, u64)> {
        // Note: Node should use a well-known index id
        for (ref event_type_id, ref event_data) in self.application_events.iter() {
            if let EventTypeIdentifier(
                Emitter::Function(node_id, TypedModuleId::ObjectState, ..)
                | Emitter::Method(node_id, TypedModuleId::ObjectState),
                ..,
            ) = event_type_id
            {
                if node_id == EPOCH_MANAGER_PACKAGE.as_node_id()
                    || node_id.entity_type() == Some(EntityType::GlobalEpochManager)
                {
                    if let Ok(EpochChangeEvent {
                        ref epoch,
                        ref validators,
                    }) = scrypto_decode(&event_data)
                    {
                        return Some((validators.clone(), *epoch));
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
    pub result: TransactionResult,
    /// Optional execution trace, controlled by config `ExecutionConfig::execution_trace`.
    pub execution_trace: TransactionExecutionTrace,
    /// Optional resource usage trace, controlled by feature flag `resources_usage`.
    pub resources_usage: ResourcesUsage,
}

impl TransactionReceipt {
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

    pub fn expect_commit(&self, success: bool) -> &CommitResult {
        match &self.result {
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
        match &self.result {
            TransactionResult::Commit(..) => panic!("Expected rejection but was commit"),
            TransactionResult::Reject(ref r) => &r.error,
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

    pub fn expect_specific_rejection<F>(&self, f: F)
    where
        F: Fn(&RejectionError) -> bool,
    {
        match &self.result {
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

    pub fn expect_specific_failure<F>(&self, f: F)
    where
        F: Fn(&RuntimeError) -> bool,
    {
        match &self.result {
            TransactionResult::Commit(c) => match &c.outcome {
                TransactionOutcome::Success(_) => panic!("Expected failure but was success"),
                TransactionOutcome::Failure(err) => {
                    if !f(&err) {
                        panic!(
                            "Expected specific failure but was different error:\n{:?}",
                            self
                        );
                    }
                }
            },
            TransactionResult::Reject(_) => panic!("Transaction was rejected"),
            TransactionResult::Abort(..) => panic!("Transaction was aborted"),
        }
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
    pub encoder: Option<&'a Bech32Encoder>,
    pub schema_lookup_callback:
        Option<Box<dyn Fn(&EventTypeIdentifier) -> Option<(LocalTypeIndex, ScryptoSchema)> + 'a>>,
}

impl<'a> TransactionReceiptDisplayContext<'a> {
    pub fn scrypto_value_serialization_context(&self) -> ScryptoValueSerializationContext<'a> {
        ScryptoValueSerializationContext::with_optional_bech32(self.encoder)
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

impl<'a> From<&'a Bech32Encoder> for TransactionReceiptDisplayContext<'a> {
    fn from(encoder: &'a Bech32Encoder) -> Self {
        Self {
            encoder: Some(encoder),
            schema_lookup_callback: None,
        }
    }
}

impl<'a> From<Option<&'a Bech32Encoder>> for TransactionReceiptDisplayContext<'a> {
    fn from(encoder: Option<&'a Bech32Encoder>) -> Self {
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

    pub fn encoder(mut self, encoder: &'a Bech32Encoder) -> Self {
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
        let scrypto_value_serialization_context = context.scrypto_value_serialization_context();
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
                #[cfg(not(feature = "serde"))]
                display_event_with_network_context(
                    f,
                    prefix!(i, c.application_events),
                    event_type_identifier,
                    event_data,
                    context,
                )?;
                #[cfg(feature = "serde")]
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
                                .to_string(scrypto_value_serialization_context),
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
                    "\n{} Entity: {}, Address: {}, Delta: {}",
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
                    "\n{} Vault: {}, Address: {}, Delta: {}",
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
        "\n{} Emitter: {}, Local Type Index: {:?}, Data: {}",
        prefix,
        event_type_identifier
            .0
            .display(receipt_context.address_display_context()),
        event_type_identifier.1,
        event_data_value.display(receipt_context.scrypto_value_serialization_context())
    )?;
    Ok(())
}

#[cfg(feature = "serde")]
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
    let event = {
        let payload =
            SborPayloadWithSchema::<ScryptoCustomTypeExtension>::new(&event_data, local_type_index);
        let serializable = payload.serializable(SerializationContext {
            mode: SerializationMode::Invertible,
            schema: &schema,
            custom_context: receipt_context.scrypto_value_serialization_context(),
        });
        serde_json::to_string(&serializable).map_err(|_| fmt::Error)
    }?;

    // Print the event information
    write!(
        f,
        "\n{} Emitter: {}, Event: {}",
        prefix,
        event_type_identifier
            .0
            .display(receipt_context.address_display_context()),
        event
    )?;
    Ok(())
}
