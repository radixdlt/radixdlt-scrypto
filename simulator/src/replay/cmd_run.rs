use crate::replay::ledger_transaction::*;
use crate::replay::Error;
use clap::Parser;
use flate2::read::GzDecoder;
use flume;
use flume::Sender;
use radix_engine::system::bootstrap::*;
use radix_engine::transaction::{execute_transaction, CostingParameters, ExecutionConfig};
use radix_engine::types::*;
use radix_engine::vm::wasm::*;
use radix_engine::vm::{DefaultNativeVm, ScryptoVm, Vm};
use radix_engine_interface::prelude::node_modules::auth::AuthAddresses;
use radix_engine_interface::prelude::NetworkDefinition;
use radix_engine_store_interface::db_key_mapper::SpreadPrefixKeyMapper;
use radix_engine_store_interface::interface::CommittableSubstateDatabase;
use radix_engine_stores::rocks_db_with_merkle_tree::RocksDBWithMerkleTreeSubstateStore;
use rocksdb::{Direction, IteratorMode, Options, DB};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tar::Archive;
use transaction::prelude::{
    IntentHash, NotarizedTransactionHash, SignedIntentHash, SystemTransactionHash,
};
use transaction::validation::{
    NotarizedTransactionValidator, TransactionValidator, ValidationConfig,
};

/// Run transactions
#[derive(Parser, Debug)]
pub struct TxnExecute {
    /// The transaction file, in `.tar.gz` format, with entries sorted
    pub source: PathBuf,
    /// Path to a folder for storing state
    pub database_dir: PathBuf,

    /// The network to use, [mainnet | stokenet]
    #[clap(short, long)]
    pub network: Option<String>,
    /// The max version to execute
    #[clap(short, long)]
    pub max_version: Option<u64>,
}

impl TxnExecute {
    pub fn run(&self) -> Result<(), Error> {
        let network = match &self.network {
            Some(n) => NetworkDefinition::from_str(n).map_err(Error::ParseNetworkError)?,
            None => NetworkDefinition::mainnet(),
        };

        let cur_version = {
            let database = RocksDBWithMerkleTreeSubstateStore::standard(self.database_dir.clone());
            let cur_version = database.get_current_version();
            // check limit
            if cur_version >= self.max_version.unwrap_or(u64::MAX) {
                return Ok(());
            }
            cur_version
        };

        let start = std::time::Instant::now();

        let (tx, rx) = flume::bounded(10);

        let mut txn_reader = if self.source.is_file() {
            let tar_gz = File::open(&self.source).map_err(Error::IOError)?;
            let tar = GzDecoder::new(tar_gz);
            let archive = Archive::new(tar);
            TxnReader::TransactionFile(archive)
        } else {
            TxnReader::StateManagerDatabaseDir(self.source.clone())
        };

        let to_version = self.max_version.clone();
        let txn_read_thread_handle =
            thread::spawn(move || txn_reader.read(cur_version, to_version, tx));

        let mut database = RocksDBWithMerkleTreeSubstateStore::standard(self.database_dir.clone());
        let txn_write_thread_handle = thread::spawn(move || {
            let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
            let iter = rx.iter();
            for tx_payload in iter {
                // execute transaction
                let transaction = LedgerTransaction::from_payload_bytes(&tx_payload)
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
                                        scrypto_vm: &scrypto_vm,
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
                                scrypto_vm: &scrypto_vm,
                                native_vm: DefaultNativeVm::new(),
                            },
                            &CostingParameters::default(),
                            &ExecutionConfig::for_notarized_transaction(network.clone()),
                            &NotarizedTransactionValidator::new(ValidationConfig::default(
                                network.id,
                            ))
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
                                scrypto_vm: &scrypto_vm,
                                native_vm: DefaultNativeVm::new(),
                            },
                            &CostingParameters::default(),
                            &ExecutionConfig::for_system_transaction(network.clone()),
                            &tx.get_executable(),
                        );
                        receipt.expect_commit_ignore_outcome().state_updates.clone()
                    }
                };
                let database_updates =
                    state_updates.create_database_updates::<SpreadPrefixKeyMapper>();
                database.commit(&database_updates);

                let new_state_root_hash = database.get_current_root_hash();
                let new_version = database.get_current_version();

                // print progress
                if new_version < 1000 || new_version % 1000 == 0 {
                    println!("New version: {}, {}", new_version, new_state_root_hash);
                }
            }
        });

        txn_read_thread_handle.join().unwrap()?;
        txn_write_thread_handle.join().unwrap();

        {
            let duration = start.elapsed();
            let database = RocksDBWithMerkleTreeSubstateStore::standard(self.database_dir.clone());
            println!("Time elapsed: {:?}", duration);
            println!("State version: {}", database.get_current_version());
            println!("State root hash: {}", database.get_current_root_hash());
        }

        Ok(())
    }
}

enum TxnReader {
    TransactionFile(Archive<GzDecoder<File>>),
    StateManagerDatabaseDir(PathBuf),
}

impl TxnReader {
    fn read(
        &mut self,
        from_version: u64,
        to_version: Option<u64>,
        tx: Sender<Vec<u8>>,
    ) -> Result<(), Error> {
        match self {
            TxnReader::TransactionFile(archive) => {
                for entry in archive.entries().map_err(Error::IOError)? {
                    // read the entry
                    let mut entry = entry.map_err(|e| Error::IOError(e))?;
                    let tx_version = entry
                        .header()
                        .path()
                        .ok()
                        .and_then(|path| path.to_str().map(ToOwned::to_owned))
                        .and_then(|s| u64::from_str(&s).ok())
                        .ok_or(Error::InvalidTransactionFile)?;
                    let mut tx_payload = Vec::new();
                    entry
                        .read_to_end(&mut tx_payload)
                        .map_err(|e| Error::IOError(e))?;

                    if tx_version <= from_version {
                        continue;
                    }
                    if let Some(to_version) = to_version {
                        if tx_version > to_version {
                            break;
                        }
                    }

                    tx.send(tx_payload).unwrap();
                }
            }
            TxnReader::StateManagerDatabaseDir(db_dir) => {
                let temp_dir = tempfile::tempdir().map_err(Error::IOError)?;

                let db = DB::open_cf_as_secondary(
                    &Options::default(),
                    db_dir.as_path(),
                    temp_dir.as_ref(),
                    vec![
                        "raw_ledger_transactions",
                        "committed_transaction_identifiers",
                    ],
                )
                .unwrap();

                let iter_start_state_version = from_version + 1;

                loop {
                    db.try_catch_up_with_primary()
                        .expect("DB catch up with primary failed");
                    let mut txn_iter = db.iterator_cf(
                        &db.cf_handle("raw_ledger_transactions").unwrap(),
                        IteratorMode::From(
                            &iter_start_state_version.to_be_bytes(),
                            Direction::Forward,
                        ),
                    );
                    while let Some(next_txn) = txn_iter.next() {
                        let next_txn = next_txn.unwrap();
                        tx.send(next_txn.1.to_vec()).unwrap();
                    }
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }

        Ok(())
    }
}
