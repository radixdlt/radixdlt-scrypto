use super::ledger_transaction_execution::execute_ledger_transaction;
use super::txn_reader::TxnReader;
use super::Error;
use clap::Parser;
use flate2::read::GzDecoder;
use flume;
use radix_engine::types::*;
use radix_engine::vm::wasm::*;
use radix_engine::vm::ScryptoVm;
use radix_engine_interface::prelude::NetworkDefinition;
use std::fs::File;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use substate_store_impls::hash_tree_support::HashTreeUpdatingDatabase;
use substate_store_impls::memory_db::InMemorySubstateDatabase;
use substate_store_interface::db_key_mapper::SpreadPrefixKeyMapper;
use substate_store_interface::interface::CommittableSubstateDatabase;
use tar::Archive;

/// Run transactions in archive, using in-memory database
#[derive(Parser, Debug)]
pub struct TxnExecuteInMemory {
    /// The transaction file, in `.tar.gz` format, with entries sorted
    pub source: PathBuf,

    /// The network to use, [mainnet | stokenet]
    #[clap(short, long)]
    pub network: Option<String>,
    /// The max version to execute
    #[clap(short, long)]
    pub max_version: Option<u64>,

    /// State hash breakpoints, in format of comma separated `<version>:<hash>`
    #[clap(short, long)]
    pub breakpoints: Option<String>,

    /// Trace transaction execution
    #[clap(long)]
    pub trace: bool,
}

impl TxnExecuteInMemory {
    pub fn run(&self) -> Result<(), Error> {
        let network = match &self.network {
            Some(n) => NetworkDefinition::from_str(n).map_err(Error::ParseNetworkError)?,
            None => NetworkDefinition::mainnet(),
        };
        let mut breakpoints = BTreeMap::<u64, Hash>::new();
        if let Some(bps) = &self.breakpoints {
            for bp in bps.split(",") {
                let mut tokens = bp.trim().split(":");
                if let Some(version) = tokens.next().and_then(|x| u64::from_str(x).ok()) {
                    if let Some(hash) = tokens.next().and_then(|x| Hash::from_str(x).ok()) {
                        if tokens.next().is_none() {
                            breakpoints.insert(version, hash);
                            continue;
                        }
                    }
                }
                return Err(Error::InvalidBreakpoints(bps.clone()));
            }
        }

        let cur_version = 0;
        let to_version = self.max_version.clone();

        let start = std::time::Instant::now();
        let (tx, rx) = flume::bounded(10);

        // txn reader
        let mut txn_reader = if self.source.is_file() {
            let tar_gz = File::open(&self.source).map_err(Error::IOError)?;
            let tar = GzDecoder::new(tar_gz);
            let archive = Archive::new(tar);
            TxnReader::TransactionFile(archive)
        } else if self.source.is_dir() {
            TxnReader::StateManagerDatabaseDir(self.source.clone())
        } else {
            return Err(Error::InvalidTransactionSource);
        };
        let txn_read_thread_handle =
            thread::spawn(move || txn_reader.read(cur_version, to_version, tx));

        // txn executor
        let substate_database = InMemorySubstateDatabase::standard();
        let mut database = HashTreeUpdatingDatabase::new(substate_database);
        let trace = self.trace;
        let txn_write_thread_handle = thread::spawn(move || {
            let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
            let iter = rx.iter();
            for tx_payload in iter {
                let state_updates = execute_ledger_transaction(
                    &database,
                    &scrypto_vm,
                    &network,
                    &tx_payload,
                    trace,
                );
                let database_updates =
                    state_updates.create_database_updates::<SpreadPrefixKeyMapper>();
                database.commit(&database_updates);

                let new_state_root_hash = database.get_current_root_hash();
                let new_version = database.get_current_version();

                if let Some(expected) = breakpoints.get(&new_version) {
                    if new_state_root_hash != *expected {
                        panic!(
                            "Unexpected state hash at version {}: expected = {}, actual = {}",
                            new_version, expected, new_state_root_hash
                        )
                    }
                }

                if new_version < 1000 || new_version % 1000 == 0 {
                    print_progress(start.elapsed(), new_version, new_state_root_hash);
                }
            }

            let duration = start.elapsed();
            println!("Time elapsed: {:?}", duration);
            println!("State version: {}", database.get_current_version());
            println!("State root hash: {}", database.get_current_root_hash());
        });

        txn_read_thread_handle.join().unwrap()?;
        txn_write_thread_handle.join().unwrap();

        Ok(())
    }
}

fn print_progress(duration: Duration, new_version: u64, new_root: Hash) {
    let seconds = duration.as_secs() % 60;
    let minutes = (duration.as_secs() / 60) % 60;
    let hours = (duration.as_secs() / 60) / 60;
    println!(
        "New version: {}, {}, {:0>2}:{:0>2}:{:0>2}",
        new_version, new_root, hours, minutes, seconds
    );
}
