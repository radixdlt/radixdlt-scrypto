use colored::*;
use scrypto::core::NetworkDefinition;
use transaction::model::*;

use crate::engine::{RejectionError, ResourceChange, RuntimeError};
use crate::fee::FeeSummary;
use crate::state_manager::StateDiff;
use crate::types::*;

#[derive(Debug)]
pub struct TransactionContents {
    pub instructions: Vec<ExecutableInstruction>,
}

#[derive(Debug)]
pub struct TransactionExecution {
    pub execution_time: Option<u128>,
    pub fee_summary: FeeSummary,
    pub application_logs: Vec<(Level, String)>,
}

/// Captures whether a transaction should be committed, and its other results
#[derive(Debug)]
pub enum TransactionResult {
    Commit(CommitResult),
    Reject(RejectResult),
}

#[derive(Debug)]
pub struct CommitResult {
    pub outcome: TransactionOutcome,
    pub state_updates: StateDiff,
    pub entity_changes: EntityChanges,
    pub resource_changes: Vec<ResourceChange>,
}

/// Captures whether a transaction's commit outcome is Success or Failure
#[derive(Debug)]
pub enum TransactionOutcome {
    Success(Vec<Vec<u8>>),
    Failure(RuntimeError),
}

impl TransactionOutcome {
    pub fn is_success(&self) -> bool {
        match self {
            Self::Success(..) => true,
            Self::Failure(..) => false,
        }
    }
}

/// A flattened combination of the transaction's result and outcome
#[derive(Debug)]
pub enum TransactionStatus<'a> {
    Success(&'a Vec<Vec<u8>>),
    Failure(&'a RuntimeError),
    Rejection(&'a RejectionError),
}

impl TransactionStatus<'_> {
    pub fn is_success(&self) -> bool {
        match self {
            Self::Success(..) => true,
            Self::Failure(..) => false,
            Self::Rejection(..) => false,
        }
    }

    pub fn is_failure(&self) -> bool {
        match self {
            Self::Success(..) => false,
            Self::Failure(..) => true,
            Self::Rejection(..) => false,
        }
    }

    pub fn is_rejection(&self) -> bool {
        match self {
            Self::Success(..) => false,
            Self::Failure(..) => false,
            Self::Rejection(..) => true,
        }
    }
}

impl TransactionResult {
    pub fn is_success(&self) -> bool {
        self.get_status().is_success()
    }

    pub fn is_failure(&self) -> bool {
        self.get_status().is_failure()
    }

    pub fn is_rejection(&self) -> bool {
        self.get_status().is_rejection()
    }

    pub fn get_status(&self) -> TransactionStatus {
        match self {
            TransactionResult::Commit(commit) => match &commit.outcome {
                TransactionOutcome::Success(x) => TransactionStatus::Success(x),
                TransactionOutcome::Failure(e) => TransactionStatus::Failure(e),
            },
            TransactionResult::Reject(rejection) => TransactionStatus::Rejection(&rejection.error),
        }
    }

    pub fn get_commit_result(&self) -> Option<&CommitResult> {
        match self {
            TransactionResult::Commit(commit) => Some(commit),
            TransactionResult::Reject(..) => None,
        }
    }
}

#[derive(Debug)]
pub struct EntityChanges {
    pub new_package_addresses: Vec<PackageAddress>,
    pub new_component_addresses: Vec<ComponentAddress>,
    pub new_resource_addresses: Vec<ResourceAddress>,
}

#[derive(Debug)]
pub struct RejectResult {
    pub error: RejectionError,
}

/// Represents a transaction receipt.
pub struct TransactionReceipt {
    pub contents: TransactionContents,
    pub execution: Option<TransactionExecution>,
    pub result: TransactionResult,
}

impl TransactionReceipt {
    pub fn expect_success(&self) -> &Vec<Vec<u8>> {
        match self.result.get_status() {
            TransactionStatus::Success(x) => &x,
            TransactionStatus::Failure(err) => {
                panic!("Expected success but was failed:\n{:?}", err)
            }
            TransactionStatus::Rejection(err) => {
                panic!("Expected success but was rejection:\n{:?}", err)
            }
        }
    }

    pub fn expect_failure<F>(&self, f: F)
    where
        F: FnOnce(&RuntimeError) -> bool,
    {
        let failure_error = match self.result.get_status() {
            TransactionStatus::Success(..) => panic!("Expected failure but was success"),
            TransactionStatus::Failure(err) => err,
            TransactionStatus::Rejection(err) => {
                panic!("Expected failure but was rejection:\n{:?}", err)
            }
        };

        if !f(&failure_error) {
            panic!(
                "Expected specific failure but was different error:\n{:?}",
                self
            );
        }
    }

    pub fn expect_rejection(&self) -> &RejectionError {
        match self.result.get_status() {
            TransactionStatus::Success(..) => panic!("Expected rejection but was success"),
            TransactionStatus::Failure(err) => {
                panic!("Expected rejection but was failed:\n{:?}", err)
            }
            TransactionStatus::Rejection(err) => err,
        }
    }

    pub fn expect_commit(&self) -> &CommitResult {
        self.result
            .get_commit_result()
            .expect("The transaction was not set to be committed")
    }

    pub fn expect_executed(&self) -> &TransactionExecution {
        self.execution
            .as_ref()
            .expect("The transaction was not executed")
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
        let result = &self.result;
        let execution = self.execution.as_ref();
        let commit = result.get_commit_result();

        write!(
            f,
            "{} {}",
            "Transaction Status:".bold().green(),
            match result.get_status() {
                TransactionStatus::Success(..) => "COMMITTED SUCCESS".blue(),
                TransactionStatus::Failure(e) => format!("COMMITTED FAILURE: {}", e).red(),
                TransactionStatus::Rejection(e) => format!("REJECTION: {}", e).red(),
            },
        )?;

        if let Some(fee_summary) = execution.map(|e| &e.fee_summary) {
            write!(
                f,
                "\n{} {} XRD burned, {} XRD tipped to validators",
                "Transaction Fee:".bold().green(),
                fee_summary.burned,
                fee_summary.tipped,
            )?;

            write!(
                f,
                "\n{} {} limit, {} consumed, {} XRD per cost unit",
                "Cost Units:".bold().green(),
                fee_summary.cost_unit_limit,
                fee_summary.cost_unit_consumed,
                fee_summary.cost_unit_price,
            )?;
        }

        write!(
            f,
            "\n{} {} ms",
            "Execution Time:".bold().green(),
            execution
                .and_then(|v| v.execution_time)
                .map(|v| v.to_string())
                .unwrap_or(String::from("?"))
        )?;

        // TODO - Need to fix the hardcoding of local simulator HRPs for transaction receipts, and for address formatting
        let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::local_simulator());
        let instructions = &self.contents.instructions;

        write!(f, "\n{}", "Instructions:".bold().green())?;
        for (i, inst) in instructions.iter().enumerate() {
            write!(
                f,
                "\n{} {}",
                prefix!(i, instructions),
                match inst {
                    ExecutableInstruction::CallFunction {
                        package_address,
                        blueprint_name,
                        method_name,
                        args,
                    } => format!(
                        "CallFunction {{ package_address: {}, blueprint_name: {:?}, method_name: {:?}, args: {:?} }}",
                        bech32_encoder.encode_package_address(&package_address),
                        blueprint_name,
                        method_name,
                        ScryptoValue::from_slice(&args).expect("Failed parse call data")
                    ),
                    ExecutableInstruction::CallMethod {
                        component_address,
                        method_name,
                        args,
                    } => format!(
                        "CallMethod {{ component_address: {}, method_name: {:?}, call_data: {:?} }}",
                        bech32_encoder.encode_component_address(&component_address),
                        method_name,
                        ScryptoValue::from_slice(&args).expect("Failed to parse call data")
                    ),
                    ExecutableInstruction::PublishPackage { .. } => "PublishPackage {..}".to_owned(),
                    i @ _ => format!("{:?}", i),
                }
            )?;
        }

        if let TransactionStatus::Success(outputs) = &result.get_status() {
            write!(f, "\n{}", "Instruction Outputs:".bold().green())?;
            for (i, output) in outputs.iter().enumerate() {
                write!(
                    f,
                    "\n{} {:?}",
                    prefix!(i, outputs),
                    ScryptoValue::from_slice(output).expect("Failed to parse return data")
                )?;
            }
        }

        if let Some(application_logs) = execution.map(|e| &e.application_logs) {
            write!(f, "\n{} {}", "Logs:".bold().green(), application_logs.len())?;
            for (i, (level, msg)) in application_logs.iter().enumerate() {
                let (l, m) = match level {
                    Level::Error => ("ERROR".red(), msg.red()),
                    Level::Warn => ("WARN".yellow(), msg.yellow()),
                    Level::Info => ("INFO".green(), msg.green()),
                    Level::Debug => ("DEBUG".cyan(), msg.cyan()),
                    Level::Trace => ("TRACE".normal(), msg.normal()),
                };
                write!(f, "\n{} [{:5}] {}", prefix!(i, application_logs), l, m)?;
            }
        }

        if let Some(entity_changes) = commit.map(|c| &c.entity_changes) {
            write!(
                f,
                "\n{} {}",
                "New Entities:".bold().green(),
                entity_changes.new_package_addresses.len()
                    + entity_changes.new_component_addresses.len()
                    + entity_changes.new_resource_addresses.len()
            )?;

            for (i, package_address) in entity_changes.new_package_addresses.iter().enumerate() {
                write!(
                    f,
                    "\n{} Package: {}",
                    prefix!(i, entity_changes.new_package_addresses),
                    bech32_encoder.encode_package_address(package_address)
                )?;
            }
            for (i, component_address) in entity_changes.new_component_addresses.iter().enumerate()
            {
                write!(
                    f,
                    "\n{} Component: {}",
                    prefix!(i, entity_changes.new_component_addresses),
                    bech32_encoder.encode_component_address(component_address)
                )?;
            }
            for (i, resource_address) in entity_changes.new_resource_addresses.iter().enumerate() {
                write!(
                    f,
                    "\n{} Resource: {}",
                    prefix!(i, entity_changes.new_resource_addresses),
                    bech32_encoder.encode_resource_address(resource_address)
                )?;
            }
        }

        Ok(())
    }
}
