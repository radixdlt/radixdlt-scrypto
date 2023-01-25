use colored::*;
use radix_engine_interface::address::{AddressDisplayContext, NO_NETWORK};
use radix_engine_interface::api::types::{GlobalAddress, Level};
use radix_engine_interface::data::{IndexedScryptoValue, ScryptoDecode};
use radix_engine_interface::model::*;
use transaction::manifest::decompiler::DecompilationContext;
use utils::ContextualDisplay;

use crate::engine::{RejectionError, RuntimeError, TrackedEvent};
use crate::fee::FeeSummary;
use crate::model::*;
use crate::state_manager::StateDiff;
use crate::types::*;

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct TransactionExecution {
    pub fee_summary: FeeSummary,
    pub events: Vec<TrackedEvent>,
}

/// Captures whether a transaction should be committed, and its other results
#[derive(Debug, Clone)]
pub enum TransactionResult {
    Commit(CommitResult),
    Reject(RejectResult),
    Abort(AbortResult),
}

impl TransactionResult {
    pub fn expect_commit(&self) -> &CommitResult {
        match self {
            TransactionResult::Commit(c) => c,
            TransactionResult::Reject(_) => panic!("Transaction was rejected"),
            TransactionResult::Abort(_) => panic!("Transaction was aborted"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommitResult {
    pub outcome: TransactionOutcome,
    pub state_updates: StateDiff,
    pub entity_changes: EntityChanges,
    pub resource_changes: Vec<ResourceChange>,
    pub application_logs: Vec<(Level, String)>,
    pub next_epoch: Option<(BTreeMap<ComponentAddress, Validator>, u64)>,
}

/// Captures whether a transaction's commit outcome is Success or Failure
#[derive(Debug, Clone)]
pub enum TransactionOutcome {
    Success(Vec<InstructionOutput>),
    Failure(RuntimeError),
}

impl TransactionOutcome {
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

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct EntityChanges {
    pub new_package_addresses: Vec<PackageAddress>,
    pub new_component_addresses: Vec<ComponentAddress>,
    pub new_resource_addresses: Vec<ResourceAddress>,
}

impl EntityChanges {
    pub fn new(new_global_addresses: Vec<GlobalAddress>) -> Self {
        let mut entity_changes = Self {
            new_package_addresses: Vec::new(),
            new_component_addresses: Vec::new(),
            new_resource_addresses: Vec::new(),
        };

        for new_global_address in new_global_addresses {
            match new_global_address {
                GlobalAddress::Package(package_address) => {
                    entity_changes.new_package_addresses.push(package_address)
                }
                GlobalAddress::Component(component_address) => entity_changes
                    .new_component_addresses
                    .push(component_address),
                GlobalAddress::Resource(resource_address) => {
                    entity_changes.new_resource_addresses.push(resource_address)
                }
            }
        }

        entity_changes
    }
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct RejectResult {
    pub error: RejectionError,
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AbortResult {
    pub reason: AbortReason,
}

#[derive(Debug, Clone, Display, PartialEq, Eq, Encode, Decode, Categorize)]
pub enum AbortReason {
    ConfiguredAbortTriggeredOnFeeLoanRepayment,
}

/// Represents a transaction receipt.
#[derive(Clone)]
pub struct TransactionReceipt {
    pub execution: TransactionExecution, // THIS FIELD IS USEFUL FOR DEBUGGING EVEN IF THE TRANSACTION IS REJECTED
    pub result: TransactionResult,
}

impl TransactionReceipt {
    pub fn is_commit(&self) -> bool {
        matches!(self.result, TransactionResult::Commit(_))
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

    pub fn expect_commit(&self) -> &CommitResult {
        match &self.result {
            TransactionResult::Commit(c) => c,
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
            InstructionOutput::Native(native) => {
                // TODO: Use downcast
                IndexedScryptoValue::from_typed(&native.as_ref())
                    .as_typed()
                    .expect("Wrong native instruction output type!")
            }
            InstructionOutput::Scrypto(value) => value
                .as_typed()
                .expect("Wrong scrypto instruction output type!"),
        }
    }

    pub fn new_package_addresses(&self) -> &Vec<PackageAddress> {
        let commit = self.expect_commit();
        &commit.entity_changes.new_package_addresses
    }

    pub fn new_component_addresses(&self) -> &Vec<ComponentAddress> {
        let commit = self.expect_commit();
        &commit.entity_changes.new_component_addresses
    }

    pub fn new_resource_addresses(&self) -> &Vec<ResourceAddress> {
        let commit = self.expect_commit();
        &commit.entity_changes.new_resource_addresses
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
        let execution = &self.execution;
        let result = &self.result;

        let bech32_encoder = context.encoder;

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

        write!(
            f,
            "\n{} {} XRD used for execution, {} XRD used for royalty, {} XRD in bad debt",
            "Transaction Fee:".bold().green(),
            execution.fee_summary.total_execution_cost_xrd,
            execution.fee_summary.total_royalty_cost_xrd,
            execution.fee_summary.bad_debt_xrd,
        )?;

        write!(
            f,
            "\n{} {} limit, {} consumed, {} XRD per cost unit, {}% tip",
            "Cost Units:".bold().green(),
            execution.fee_summary.cost_unit_limit,
            execution.fee_summary.cost_unit_consumed,
            execution.fee_summary.cost_unit_price,
            execution.fee_summary.tip_percentage
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
        }

        let decompilation_context = DecompilationContext::new_with_optional_network(bech32_encoder);

        if let TransactionResult::Commit(c) = &result {
            if let TransactionOutcome::Success(outputs) = &c.outcome {
                write!(f, "\n{}", "Outputs:".bold().green())?;
                for (i, output) in outputs.iter().enumerate() {
                    write!(
                        f,
                        "\n{} {}",
                        prefix!(i, outputs),
                        IndexedScryptoValue::from_slice(&output.as_vec())
                            .expect("Failed to parse return data")
                            .display(decompilation_context.for_value_display())
                    )?;
                }
            }
        }

        if let TransactionResult::Commit(c) = &result {
            write!(
                f,
                "\n{} {}",
                "New Entities:".bold().green(),
                c.entity_changes.new_package_addresses.len()
                    + c.entity_changes.new_component_addresses.len()
                    + c.entity_changes.new_resource_addresses.len()
            )?;

            for (i, package_address) in c.entity_changes.new_package_addresses.iter().enumerate() {
                write!(
                    f,
                    "\n{} Package: {}",
                    prefix!(i, c.entity_changes.new_package_addresses),
                    package_address.display(bech32_encoder)
                )?;
            }
            for (i, component_address) in
                c.entity_changes.new_component_addresses.iter().enumerate()
            {
                write!(
                    f,
                    "\n{} Component: {}",
                    prefix!(i, c.entity_changes.new_component_addresses),
                    component_address.display(bech32_encoder)
                )?;
            }
            for (i, resource_address) in c.entity_changes.new_resource_addresses.iter().enumerate()
            {
                write!(
                    f,
                    "\n{} Resource: {}",
                    prefix!(i, c.entity_changes.new_resource_addresses),
                    resource_address.display(bech32_encoder)
                )?;
            }
        }

        Ok(())
    }
}
