use colored::*;
use scrypto::address::ContextualDisplay;
use transaction::manifest::decompiler::{decompile_instruction, DecompilationContext};
use transaction::model::*;

use crate::engine::{RejectionError, ResourceChange, RuntimeError};
use crate::fee::FeeSummary;
use crate::state_manager::StateDiff;
use crate::types::*;

#[derive(Debug, TypeId, Encode, Decode)]
pub struct TransactionContents {
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct TransactionExecution {
    pub fee_summary: FeeSummary,
    pub application_logs: Vec<(Level, String)>,
}

/// Captures whether a transaction should be committed, and its other results
#[derive(Debug, TypeId, Encode, Decode)]
pub enum TransactionResult {
    Commit(CommitResult),
    Reject(RejectResult),
}

impl TransactionResult {
    pub fn expect_commit(self) -> CommitResult {
        match self {
            TransactionResult::Commit(c) => c,
            TransactionResult::Reject(_) => panic!("Transaction was rejected"),
        }
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CommitResult {
    pub outcome: TransactionOutcome,
    pub state_updates: StateDiff,
    pub entity_changes: EntityChanges,
    pub resource_changes: Vec<ResourceChange>,
}

/// Captures whether a transaction's commit outcome is Success or Failure
#[derive(Debug, TypeId, Encode, Decode)]
pub enum TransactionOutcome {
    Success(Vec<Vec<u8>>),
    Failure(RuntimeError),
}

impl TransactionOutcome {
    pub fn expect_success(self) -> Vec<Vec<u8>> {
        match self {
            TransactionOutcome::Success(results) => results,
            TransactionOutcome::Failure(error) => panic!("Outcome was a failure: {}", error),
        }
    }

    pub fn success_or_else<E, F: FnOnce(RuntimeError) -> E>(self, f: F) -> Result<Vec<Vec<u8>>, E> {
        match self {
            TransactionOutcome::Success(results) => Ok(results),
            TransactionOutcome::Failure(error) => Err(f(error)),
        }
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct EntityChanges {
    pub new_package_addresses: Vec<PackageAddress>,
    pub new_component_addresses: Vec<ComponentAddress>,
    pub new_resource_addresses: Vec<ResourceAddress>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct RejectResult {
    pub error: RejectionError,
}

/// Represents a transaction receipt.
#[derive(TypeId, Encode, Decode)]
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

    pub fn expect_rejection(&self) -> &RejectionError {
        match &self.result {
            TransactionResult::Commit(..) => panic!("Expected rejection but was commit"),
            TransactionResult::Reject(ref r) => &r.error,
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
            TransactionResult::Reject(err) => panic!("Transaction was rejected:\n{:?}", err),
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
                            "Expected specific failure but was different error:\n{}",
                            self.displayable(None)
                        );
                    }
                }
            },
            TransactionResult::Reject(_) => panic!("Transaction was rejected"),
        }
    }

    pub fn output<T: Decode>(&self, nth: usize) -> T {
        scrypto_decode::<T>(&self.expect_commit_success()[nth][..])
            .expect("Wrong instruction output type!")
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

    pub fn displayable<'a, T: Into<Option<&'a Bech32Encoder>>>(
        &'a self,
        bech32_encoder: T,
    ) -> DisplayableTransactionReceipt<'a> {
        DisplayableTransactionReceipt(self, bech32_encoder.into())
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
        write!(f, "{}", self.displayable(None))
    }
}

pub struct DisplayableTransactionReceipt<'a>(&'a TransactionReceipt, Option<&'a Bech32Encoder>);

impl<'a> fmt::Display for DisplayableTransactionReceipt<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let contents = &self.0.contents;
        let execution = &self.0.execution;
        let result = &self.0.result;

        let bech32_encoder = self.1;

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

        let mut decompilation_context =
            DecompilationContext::new_with_optional_network(bech32_encoder);

        write!(f, "\n{}", "Instructions:".bold().green())?;
        for (i, inst) in contents.instructions.iter().enumerate() {
            write!(f, "\n{} ", prefix!(i, contents.instructions))?;
            let res = decompile_instruction(f, inst, &mut decompilation_context);
            if let Err(err) = res {
                write!(f, "[INVALID_INSTRUCTION({:?})]", err)?
            }
        }

        if let TransactionResult::Commit(c) = &result {
            if let TransactionOutcome::Success(outputs) = &c.outcome {
                write!(f, "\n{}", "Instruction Outputs:".bold().green())?;
                for (i, output) in outputs.iter().enumerate() {
                    write!(
                        f,
                        "\n{} {}",
                        prefix!(i, outputs),
                        ScryptoValue::from_slice(output)
                            .expect("Failed to parse return data")
                            .displayable(
                                bech32_encoder,
                                &decompilation_context.bucket_names,
                                &decompilation_context.proof_names
                            )
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

impl<'a> fmt::Debug for DisplayableTransactionReceipt<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
