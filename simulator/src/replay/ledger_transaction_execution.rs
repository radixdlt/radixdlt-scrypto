use super::ledger_transaction::*;
use radix_engine::system::bootstrap::*;
use radix_engine::track::StateUpdates;
use radix_engine::transaction::{execute_transaction, CostingParameters, ExecutionConfig};
use radix_engine::types::*;
use radix_engine::vm::wasm::*;
use radix_engine::vm::{DefaultNativeVm, ScryptoVm, Vm};
use radix_engine_interface::prelude::node_modules::auth::AuthAddresses;
use radix_engine_interface::prelude::NetworkDefinition;
use radix_engine_store_interface::interface::SubstateDatabase;
use transaction::validation::{
    NotarizedTransactionValidator, TransactionValidator, ValidationConfig,
};

pub fn execute_ledger_transaction<S: SubstateDatabase>(
    database: &S,
    scrypto_vm: &ScryptoVm<DefaultWasmEngine>,
    network: &NetworkDefinition,
    tx_payload: &[u8],
) -> StateUpdates {
    let transaction =
        LedgerTransaction::from_payload_bytes(&tx_payload).expect("Failed to decode transaction");
    let prepared = transaction
        .prepare()
        .expect("Failed to prepare transaction");
    match &prepared.inner {
        PreparedLedgerTransactionInner::Genesis(prepared_genesis_tx) => {
            match prepared_genesis_tx.as_ref() {
                PreparedGenesisTransaction::Flash(_) => {
                    let receipt = create_substate_flash_for_genesis();
                    receipt.state_updates
                }
                PreparedGenesisTransaction::Transaction(tx) => {
                    let receipt = execute_transaction(
                        database,
                        Vm {
                            scrypto_vm,
                            native_vm: DefaultNativeVm::new(),
                        },
                        &CostingParameters::default(),
                        &ExecutionConfig::for_genesis_transaction(network.clone()),
                        &tx.get_executable(btreeset!(AuthAddresses::system_role())),
                    );
                    receipt.into_commit_ignore_outcome().state_updates
                }
            }
        }
        PreparedLedgerTransactionInner::UserV1(tx) => {
            let receipt = execute_transaction(
                database,
                Vm {
                    scrypto_vm,
                    native_vm: DefaultNativeVm::new(),
                },
                &CostingParameters::default(),
                &ExecutionConfig::for_notarized_transaction(network.clone()),
                &NotarizedTransactionValidator::new(ValidationConfig::default(network.id))
                    .validate(tx.as_ref().clone())
                    .expect("Transaction validation failure")
                    .get_executable(),
            );
            receipt.into_commit_ignore_outcome().state_updates
        }
        PreparedLedgerTransactionInner::RoundUpdateV1(tx) => {
            let receipt = execute_transaction(
                database,
                Vm {
                    scrypto_vm,
                    native_vm: DefaultNativeVm::new(),
                },
                &CostingParameters::default(),
                &ExecutionConfig::for_system_transaction(network.clone()),
                &tx.get_executable(),
            );
            receipt.into_commit_ignore_outcome().state_updates
        }
    }
}
