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

pub fn execute_ledger_transaction<S: SubstateDatabase>(
    database: &S,
    vm_modules: &impl VmInitialize,
    network: &NetworkDefinition,
    tx_payload: &[u8],
    trace: bool,
) -> StateUpdates {
    let prepared = prepare_ledger_transaction(tx_payload);
    execute_prepared_ledger_transaction(database, vm_modules, network, &prepared, trace)
        .into_state_updates()
}

pub fn prepare_ledger_transaction(tx_payload: &[u8]) -> PreparedLedgerTransaction {
    let transaction =
        LedgerTransaction::from_payload_bytes(&tx_payload).expect("Failed to decode transaction");
    let prepared = transaction
        .prepare()
        .expect("Failed to prepare transaction");
    prepared
}

pub fn execute_prepared_ledger_transaction<S: SubstateDatabase>(
    database: &S,
    vm_modules: &impl VmInitialize,
    network: &NetworkDefinition,
    prepared: &PreparedLedgerTransaction,
    trace: bool,
) -> LedgerTransactionReceipt {
    match &prepared.inner {
        PreparedLedgerTransactionInner::Genesis(prepared_genesis_tx) => {
            match prepared_genesis_tx.as_ref() {
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
                        tx.get_executable(btreeset!(system_execution(SystemExecution::Protocol))),
                    );
                    LedgerTransactionReceipt::Standard(receipt)
                }
            }
        }
        PreparedLedgerTransactionInner::UserV1(tx) => {
            let receipt = execute_transaction(
                database,
                vm_modules,
                &ExecutionConfig::for_notarized_transaction(network.clone())
                    .with_kernel_trace(trace)
                    .with_cost_breakdown(trace),
                NotarizedTransactionValidatorV1::new(ValidationConfig::default(network.id))
                    .validate(tx.as_ref().clone())
                    .expect("Transaction validation failure")
                    .get_executable(),
            );
            LedgerTransactionReceipt::Standard(receipt)
        }
        PreparedLedgerTransactionInner::RoundUpdateV1(tx) => {
            let receipt = execute_transaction(
                database,
                vm_modules,
                &ExecutionConfig::for_system_transaction(network.clone())
                    .with_kernel_trace(trace)
                    .with_cost_breakdown(trace),
                tx.get_executable(),
            );
            LedgerTransactionReceipt::Standard(receipt)
        }
        PreparedLedgerTransactionInner::FlashV1(tx) => {
            LedgerTransactionReceipt::ProtocolUpdateFlash(tx.state_updates.clone())
        }
        PreparedLedgerTransactionInner::UserV2(tx) => {
            let receipt = execute_transaction(
                database,
                vm_modules,
                &ExecutionConfig::for_notarized_transaction(network.clone())
                    .with_kernel_trace(trace)
                    .with_cost_breakdown(trace),
                NotarizedTransactionValidatorV2::new(ValidationConfig::default(network.id))
                    .validate(tx.as_ref().clone())
                    .expect("Transaction validation failure")
                    .get_executable(),
            );
            LedgerTransactionReceipt::Standard(receipt)
        }
    }
}
