use radix_common::prelude::*;
use radix_engine::system::bootstrap::*;
use radix_engine::transaction::{
    execute_transaction, ExecutionConfig, TransactionFeeSummary, TransactionReceipt,
};
use radix_engine::vm::*;
use radix_engine_interface::prelude::system_execution;
use radix_substate_store_interface::interface::SubstateDatabase;
use radix_transactions::prelude::*;
use radix_transactions::validation::*;

pub enum LedgerTransactionReceipt {
    Flash(FlashReceipt),
    Standard(TransactionReceipt),
    ProtocolUpdateFlash(StateUpdates),
}

impl LedgerTransactionReceipt {
    pub fn into_state_updates(self) -> StateUpdates {
        match self {
            LedgerTransactionReceipt::Flash(receipt) => receipt.state_updates,
            LedgerTransactionReceipt::Standard(receipt) => {
                receipt.into_commit_ignore_outcome().state_updates
            }
            LedgerTransactionReceipt::ProtocolUpdateFlash(state_updates) => state_updates,
        }
    }

    pub fn fee_summary(&self) -> Option<&TransactionFeeSummary> {
        match self {
            LedgerTransactionReceipt::Flash(_) => None,
            LedgerTransactionReceipt::Standard(receipt) => Some(&receipt.fee_summary),
            LedgerTransactionReceipt::ProtocolUpdateFlash(_) => None,
        }
    }
}

pub enum LedgerTransactionKindedHash {
    Genesis(SystemTransactionHash),
    User(NotarizedTransactionHash),
    Validator(Hash),
    ProtocolUpdate(Hash),
}

pub fn execute_ledger_transaction<S: SubstateDatabase>(
    database: &S,
    vm_modules: &impl VmInitialize,
    network: &NetworkDefinition,
    raw: &RawLedgerTransaction,
    trace: bool,
) -> (LedgerTransactionKindedHash, LedgerTransactionReceipt) {
    let validator = TransactionValidator::new(database, network);
    let validated = raw
        .validate(&validator, AcceptedLedgerTransactionKind::Any)
        .expect("Ledger transaction should be valid");

    let kinded_hash = match &validated.inner {
        ValidatedLedgerTransactionInner::Genesis(tx) => {
            LedgerTransactionKindedHash::Genesis(tx.system_transaction_hash())
        }
        ValidatedLedgerTransactionInner::User(tx) => {
            LedgerTransactionKindedHash::User(tx.notarized_transaction_hash())
        }
        ValidatedLedgerTransactionInner::Validator(tx) => {
            LedgerTransactionKindedHash::Validator(tx.summary.hash)
        }
        ValidatedLedgerTransactionInner::ProtocolUpdate(tx) => {
            LedgerTransactionKindedHash::ProtocolUpdate(tx.flash_transaction_hash().into_hash())
        }
    };

    let receipt = match validated.inner {
        ValidatedLedgerTransactionInner::Genesis(prepared_genesis_tx) => {
            match prepared_genesis_tx {
                PreparedGenesisTransaction::Flash(_) => {
                    let receipt = create_substate_flash_for_genesis();
                    LedgerTransactionReceipt::Flash(receipt)
                }
                PreparedGenesisTransaction::Transaction(tx) => {
                    let receipt = execute_transaction(
                        database,
                        vm_modules,
                        &ExecutionConfig::for_genesis_transaction(network.clone())
                            .with_kernel_trace(trace)
                            .with_cost_breakdown(trace),
                        tx.create_executable(btreeset!(system_execution(
                            SystemExecution::Protocol
                        ))),
                    );
                    LedgerTransactionReceipt::Standard(receipt)
                }
            }
        }
        ValidatedLedgerTransactionInner::User(tx) => {
            let receipt = execute_transaction(
                database,
                vm_modules,
                &ExecutionConfig::for_notarized_transaction(network.clone())
                    .with_kernel_trace(trace)
                    .with_cost_breakdown(trace),
                tx.create_executable(),
            );
            LedgerTransactionReceipt::Standard(receipt)
        }
        ValidatedLedgerTransactionInner::Validator(tx) => {
            let receipt = execute_transaction(
                database,
                vm_modules,
                &ExecutionConfig::for_system_transaction(network.clone())
                    .with_kernel_trace(trace)
                    .with_cost_breakdown(trace),
                tx.create_executable(),
            );
            LedgerTransactionReceipt::Standard(receipt)
        }
        ValidatedLedgerTransactionInner::ProtocolUpdate(tx) => {
            LedgerTransactionReceipt::ProtocolUpdateFlash(tx.state_updates)
        }
    };

    (kinded_hash, receipt)
}
