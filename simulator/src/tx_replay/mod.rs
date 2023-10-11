pub mod ledger_transaction;

use clap::Parser;
use ledger_transaction::*;
use radix_engine::system::bootstrap::*;
use radix_engine::transaction::{execute_transaction, CostingParameters, ExecutionConfig};
use radix_engine::types::*;
use radix_engine::vm::wasm::*;
use radix_engine::vm::{DefaultNativeVm, ScryptoVm, Vm};
use radix_engine_interface::prelude::node_modules::auth::AuthAddresses;
use radix_engine_store_interface::db_key_mapper::SpreadPrefixKeyMapper;
use radix_engine_store_interface::interface::CommittableSubstateDatabase;
use radix_engine_stores::hash_tree_support::HashTreeUpdatingDatabase;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use transaction::validation::{
    NotarizedTransactionValidator, TransactionValidator, ValidationConfig,
};

/// Replay transactions and output state root.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, name = "tx-replay")]
pub struct Args {
    /// The network to use when outputting manifest, [mainnet | stokenet]
    #[clap(short, long)]
    pub network: Option<String>,
    /// The transaction directory
    pub transaction_dir: String,
    /// The number of transactions to replay
    pub transaction_count: u32,
}

#[derive(Debug)]
pub enum Error {
    ParseNetworkError(ParseNetworkError),
    IOError(std::io::Error),
}

pub fn run() -> Result<(), Error> {
    let args = Args::parse();

    let network = match args.network {
        Some(n) => NetworkDefinition::from_str(&n).map_err(Error::ParseNetworkError)?,
        None => NetworkDefinition::mainnet(),
    };

    let in_memory = InMemorySubstateDatabase::standard();
    let mut database = HashTreeUpdatingDatabase::new(in_memory);

    let start = std::time::Instant::now();
    for i in 0..args.transaction_count {
        let path = format!("{}/{:0>8}", args.transaction_dir, i);
        let transaction =
            LedgerTransaction::from_payload_bytes(&std::fs::read(path).map_err(Error::IOError)?)
                .expect("Failed to decode transaction");
        let prepared = transaction
            .prepare()
            .expect("Failed to prepare transaction");
        let state_updates = match &prepared.inner {
            PreparedLedgerTransactionInner::Genesis(prepared_genesis_tx) => {
                match prepared_genesis_tx.as_ref() {
                    PreparedGenesisTransaction::Flash(_) => {
                        let receipt = create_substate_flash_for_genesis();
                        receipt.state_updates
                    }
                    PreparedGenesisTransaction::Transaction(tx) => {
                        let receipt = execute_transaction(
                            &database,
                            Vm {
                                scrypto_vm: &ScryptoVm::<DefaultWasmEngine>::default(),
                                native_vm: DefaultNativeVm::new(),
                            },
                            &CostingParameters::default(),
                            &ExecutionConfig::for_genesis_transaction(network.clone()),
                            &tx.get_executable(btreeset!(AuthAddresses::system_role())),
                        );
                        receipt.expect_commit_ignore_outcome().state_updates.clone()
                    }
                }
            }
            PreparedLedgerTransactionInner::UserV1(tx) => {
                let receipt = execute_transaction(
                    &database,
                    Vm {
                        scrypto_vm: &ScryptoVm::<DefaultWasmEngine>::default(),
                        native_vm: DefaultNativeVm::new(),
                    },
                    &CostingParameters::default(),
                    &ExecutionConfig::for_genesis_transaction(network.clone()),
                    &NotarizedTransactionValidator::new(ValidationConfig::default(network.id))
                        .validate(tx.as_ref().clone())
                        .expect("Transaction validation failure")
                        .get_executable(),
                );
                receipt.expect_commit_ignore_outcome().state_updates.clone()
            }
            PreparedLedgerTransactionInner::RoundUpdateV1(tx) => {
                let receipt = execute_transaction(
                    &database,
                    Vm {
                        scrypto_vm: &ScryptoVm::<DefaultWasmEngine>::default(),
                        native_vm: DefaultNativeVm::new(),
                    },
                    &CostingParameters::default(),
                    &ExecutionConfig::for_genesis_transaction(network.clone()),
                    &tx.get_executable(),
                );
                receipt.expect_commit_ignore_outcome().state_updates.clone()
            }
        };
        let database_updates = state_updates.create_database_updates::<SpreadPrefixKeyMapper>();
        database.commit(&database_updates);
        let new_version = database.get_current_version();
        let new_root = database.get_current_root_hash();
        println!("New version: {}, {}", new_version, new_root);
    }
    let duration = start.elapsed();
    println!("Time elapsed is: {:?}", duration);

    Ok(())
}
