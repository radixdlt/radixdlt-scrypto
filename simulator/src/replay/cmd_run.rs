use crate::replay::ledger_transaction::*;
use crate::replay::Error;
use clap::Parser;
use flate2::read::GzDecoder;
use radix_engine::system::bootstrap::*;
use radix_engine::transaction::{execute_transaction, CostingParameters, ExecutionConfig};
use radix_engine::types::*;
use radix_engine::vm::wasm::*;
use radix_engine::vm::{DefaultNativeVm, ScryptoVm, Vm};
use radix_engine_interface::prelude::node_modules::auth::AuthAddresses;
use radix_engine_interface::prelude::NetworkDefinition;
use radix_engine_store_interface::db_key_mapper::SpreadPrefixKeyMapper;
use radix_engine_store_interface::interface::CommittableSubstateDatabase;
use radix_engine_stores::hash_tree_support::HashTreeUpdatingDatabase;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use std::fs::File;
use std::io::Read;
use tar::Archive;
use transaction::validation::{
    NotarizedTransactionValidator, TransactionValidator, ValidationConfig,
};

/// Run transactions
#[derive(Parser, Debug)]
pub struct Run {
    /// The network to use when outputting manifest, [mainnet | stokenet]
    #[clap(short, long)]
    pub network: Option<String>,
    /// The transaction file, in `.tar.gz` format
    pub transaction: String,
    /// The max number of transactions to run
    pub limit: Option<u32>,
}

impl Run {
    pub fn run(&self) -> Result<(), Error> {
        let network = match &self.network {
            Some(n) => NetworkDefinition::from_str(n).map_err(Error::ParseNetworkError)?,
            None => NetworkDefinition::mainnet(),
        };

        let in_memory = InMemorySubstateDatabase::standard();
        let mut database = HashTreeUpdatingDatabase::new(in_memory);

        let tar_gz = File::open(&self.transaction).map_err(Error::IOError)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);

        let start = std::time::Instant::now();
        for entry in archive.entries().map_err(Error::IOError)? {
            let mut entry = entry.map_err(|e| Error::IOError(e))?;
            let mut buffer = Vec::new();
            entry
                .read_to_end(&mut buffer)
                .map_err(|e| Error::IOError(e))?;
            if buffer.is_empty() {
                // folders
                continue;
            }

            let transaction = LedgerTransaction::from_payload_bytes(&buffer)
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
}
