use crate::blueprints::epoch_manager::{EpochChangeEvent, Validator};
use crate::errors::*;
use crate::state_manager::StateDiff;
use crate::system::kernel_modules::costing::FeeSummary;
use crate::system::kernel_modules::execution_trace::{
    ExecutionTrace, ResourceChange, WorktopChange,
};
use crate::types::*;
use colored::*;
use radix_engine_interface::address::{AddressDisplayContext, NO_NETWORK};
use radix_engine_interface::api::types::*;
use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use radix_engine_interface::data::scrypto::{
    ScryptoDecode, ScryptoValue, ScryptoValueDisplayContext,
};
use utils::ContextualDisplay;

#[derive(Debug, Clone, Default, ScryptoSbor)]
pub struct ResourcesUsage {
    pub heap_allocations_sum: usize,
    pub heap_peak_memory: usize,
    pub cpu_cycles: u64,
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct TransactionExecutionTrace {
    pub execution_traces: Vec<ExecutionTrace>,
    pub resource_changes: IndexMap<usize, Vec<ResourceChange>>,
    pub resources_usage: ResourcesUsage,
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
    pub outcome: TransactionOutcome,
    pub fee_summary: FeeSummary,
    pub actual_fee_payments: BTreeMap<ObjectId, Decimal>,
    pub state_updates: StateDiff,
    pub application_events: Vec<(EventTypeIdentifier, Vec<u8>)>,
    pub application_logs: Vec<(Level, String)>,
}

impl CommitResult {
    pub fn next_epoch(&self) -> Option<(BTreeMap<ComponentAddress, Validator>, u64)> {
        // TODO: Simplify once ScryptoEvent trait is implemented
        let expected_event_name = {
            let (local_type_index, schema) = generate_full_schema_from_single_type::<
                EpochChangeEvent,
                ScryptoCustomTypeExtension,
            >();
            (*schema
                .resolve_type_metadata(local_type_index)
                .expect("Cant fail")
                .type_name)
                .to_owned()
        };
        self.application_events
            .iter()
            .find(|(identifier, _)| match identifier {
                EventTypeIdentifier(
                    Emitter::Function(
                        RENodeId::GlobalObject(Address::Package(EPOCH_MANAGER_PACKAGE)),
                        NodeModuleId::SELF,
                        ..,
                    )
                    | Emitter::Method(
                        RENodeId::GlobalObject(Address::Component(ComponentAddress::EpochManager(
                            ..,
                        ))),
                        NodeModuleId::SELF,
                    ),
                    event_name,
                ) if *event_name == expected_event_name => true,
                _ => false,
            })
            .map(|(_, data)| scrypto_decode::<EpochChangeEvent>(data).expect("Impossible Case!"))
            .map(|event| (event.validators, event.epoch))
    }

    pub fn new_package_addresses(&self) -> Vec<PackageAddress> {
        todo!()
    }

    pub fn new_component_addresses(&self) -> Vec<ComponentAddress> {
        todo!()
    }

    pub fn new_resource_addresses(&self) -> &Vec<ResourceAddress> {
        todo!()
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

    pub fn success_or_else<E, F: FnOnce(&RuntimeError) -> E>(
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
    pub execution_trace: TransactionExecutionTrace,
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
                    panic!("Transaction outcome (success or not) does not match")
                }
                c
            }
            TransactionResult::Reject(_) => panic!("Transaction was rejected"),
            TransactionResult::Abort(_) => panic!("Transaction was aborted"),
        }
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
        F: FnOnce(&RejectionError) -> bool,
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

    pub fn expect_commit_success(&self) -> &Vec<InstructionOutput> {
        match &self.result {
            TransactionResult::Commit(c) => match &c.outcome {
                TransactionOutcome::Success(x) => x,
                TransactionOutcome::Failure(err) => {
                    panic!("Expected success but was failed:\n{:?}", err)
                }
            },
            TransactionResult::Reject(err) => panic!("Transaction was rejected:\n{:?}", err),
            TransactionResult::Abort(..) => panic!("Transaction was aborted"),
        }
    }

    pub fn expect_commit_failure(&self) -> &RuntimeError {
        match &self.result {
            TransactionResult::Commit(c) => match &c.outcome {
                TransactionOutcome::Success(_) => {
                    panic!("Expected failure but was success")
                }
                TransactionOutcome::Failure(err) => err,
            },
            TransactionResult::Reject(_) => panic!("Transaction was rejected"),
            TransactionResult::Abort(..) => panic!("Transaction was aborted"),
        }
    }

    pub fn expect_specific_failure<F>(&self, f: F)
    where
        F: FnOnce(&RuntimeError) -> bool,
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

    pub fn output<T: ScryptoDecode>(&self, nth: usize) -> T {
        match &self.expect_commit_success()[nth] {
            InstructionOutput::CallReturn(value) => {
                scrypto_decode::<T>(value).expect("Output can't be converted")
            }
            InstructionOutput::None => panic!("No call return from the instruction"),
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
        write!(f, "{}", self.display(NO_NETWORK))
    }
}

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for TransactionReceipt {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &AddressDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        let result = &self.result;
        let bech32_encoder = context.encoder;
        let context = ScryptoValueDisplayContext::with_optional_bench32(bech32_encoder);

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

            // TODO: Pretty print the events. Perhaps with Contextual display when the event schema
            // can be looked up.
            write!(
                f,
                "\n{} {}",
                "Events:".bold().green(),
                c.application_events.len()
            )?;
            for (i, (event_type_identifier, event_data)) in c.application_events.iter().enumerate()
            {
                let event_data_value =
                    scrypto_decode::<ScryptoValue>(&event_data).expect("Event must be decodable!");
                write!(
                    f,
                    "\n{} Identifier: {:?}, Event Data: {:?}",
                    prefix!(i, c.application_events),
                    event_type_identifier,
                    event_data_value
                )?;
            }

            if let TransactionOutcome::Success(outputs) = &c.outcome {
                write!(f, "\n{}", "Outputs:".bold().green())?;
                for (i, output) in outputs.iter().enumerate() {
                    write!(
                        f,
                        "\n{} {}",
                        prefix!(i, outputs),
                        match output {
                            InstructionOutput::CallReturn(x) => IndexedScryptoValue::from_slice(&x)
                                .expect("Impossible case! Instruction output can't be decoded")
                                .to_string(context),
                            InstructionOutput::None => "None".to_string(),
                        }
                    )?;
                }
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
                    package_address.display(bech32_encoder)
                )?;
            }
            for (i, component_address) in c.new_component_addresses().iter().enumerate() {
                write!(
                    f,
                    "\n{} Component: {}",
                    prefix!(i, c.new_component_addresses()),
                    component_address.display(bech32_encoder)
                )?;
            }
            for (i, resource_address) in c.new_resource_addresses().iter().enumerate() {
                write!(
                    f,
                    "\n{} Resource: {}",
                    prefix!(i, c.new_resource_addresses()),
                    resource_address.display(bech32_encoder)
                )?;
            }
        }

        Ok(())
    }
}
