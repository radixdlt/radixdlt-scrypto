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
    pub execution: TransactionExecution, // THIS FIELD IS USEFUL FOR DEBUGGING EVEN IF THE TRANSACTION IS REJECTED
    pub result: TransactionResult,
}

impl TransactionReceipt {
    pub fn is_commit(&self) -> bool {
        matches!(self.result, TransactionResult::Commit(_))
    }

    pub fn is_rejection(&self) -> bool {
        matches!(self.result, TransactionResult::Reject(_))
    }

    pub fn expect_commit(&self) -> &CommitResult {
        match &self.result {
            TransactionResult::Commit(c) => c,
            TransactionResult::Reject(_) => panic!("Transaction was rejected"),
        }
    }

    pub fn expect_commit_success(&self) -> &Vec<Vec<u8>> {
        match &self.result {
            TransactionResult::Commit(c) => match &c.outcome {
                TransactionOutcome::Success(x) => x,
                TransactionOutcome::Failure(err) => {
                    panic!("Expected success but was failed:\n{:?}", err)
                }
            },
            TransactionResult::Reject(_) => panic!("Transaction was rejected"),
        }
    }

    pub fn expect_commit_failure<F>(&self, f: F)
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
        }
    }

    pub fn expect_rejection(&self) -> &RejectionError {
        match &self.result {
            TransactionResult::Commit(..) => panic!("Expected rejection but was commit"),
            TransactionResult::Reject(ref r) => &r.error,
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
        let contents = &self.contents;
        let execution = &self.execution;
        let result = &self.result;

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
            },
        )?;

        write!(
            f,
            "\n{} {} XRD burned, {} XRD tipped to validators",
            "Transaction Fee:".bold().green(),
            execution.fee_summary.burned,
            execution.fee_summary.tipped,
        )?;

        write!(
            f,
            "\n{} {} limit, {} consumed, {} XRD per cost unit",
            "Cost Units:".bold().green(),
            execution.fee_summary.cost_unit_limit,
            execution.fee_summary.cost_unit_consumed,
            execution.fee_summary.cost_unit_price,
        )?;

        write!(
            f,
            "\n{} {}",
            "Logs:".bold().green(),
            execution.application_logs.len()
        )?;
        for (i, (level, msg)) in execution.application_logs.iter().enumerate() {
            let (l, m) = match level {
                Level::Error => ("ERROR".red(), msg.red()),
                Level::Warn => ("WARN".yellow(), msg.yellow()),
                Level::Info => ("INFO".green(), msg.green()),
                Level::Debug => ("DEBUG".cyan(), msg.cyan()),
                Level::Trace => ("TRACE".normal(), msg.normal()),
            };
            write!(
                f,
                "\n{} [{:5}] {}",
                prefix!(i, execution.application_logs),
                l,
                m
            )?;
        }

        // TODO - Need to fix the hardcoding of local simulator HRPs for transaction receipts, and for address formatting
        let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::local_simulator());

        write!(f, "\n{}", "Instructions:".bold().green())?;
        for (i, inst) in contents.instructions.iter().enumerate() {
            write!(
                f,
                "\n{} {}",
                prefix!(i, contents.instructions),
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

        if let TransactionResult::Commit(c) = &result {
            if let TransactionOutcome::Success(outputs) = &c.outcome {
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
                    bech32_encoder.encode_package_address(package_address)
                )?;
            }
            for (i, component_address) in
                c.entity_changes.new_component_addresses.iter().enumerate()
            {
                write!(
                    f,
                    "\n{} Component: {}",
                    prefix!(i, c.entity_changes.new_component_addresses),
                    bech32_encoder.encode_component_address(component_address)
                )?;
            }
            for (i, resource_address) in c.entity_changes.new_resource_addresses.iter().enumerate()
            {
                write!(
                    f,
                    "\n{} Resource: {}",
                    prefix!(i, c.entity_changes.new_resource_addresses),
                    bech32_encoder.encode_resource_address(resource_address)
                )?;
            }
        }

        Ok(())
    }
}
