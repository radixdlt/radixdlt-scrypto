use crate::replay::ledger_transaction_execution::execute_ledger_transaction;
use crate::replay::ledger_transaction_execution::prepare_ledger_transaction;

use super::txn_reader::TxnReader;
use super::Error;
use clap::Parser;
use flate2::read::GzDecoder;
use flume;
use radix_engine::types::*;
use radix_engine::vm::wasm::*;
use radix_engine::vm::ScryptoVm;
use radix_engine_interface::prelude::NetworkDefinition;
use radix_engine_store_interface::db_key_mapper::SpreadPrefixKeyMapper;
use radix_engine_store_interface::interface::CommittableSubstateDatabase;
use radix_engine_stores::rocks_db_with_merkle_tree::RocksDBWithMerkleTreeSubstateStore;
use std::fs::File;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tar::Archive;

/// Run transactions in archive, using RocksDB
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

    /// Enables kernel trace
    #[clap(long)]
    pub kernel_trace: bool,
    /// Enables execution trace
    #[clap(long)]
    pub execution_trace: bool,
    /// Enables cost breakdown
    #[clap(long)]
    pub cost_breakdown: bool,
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
            if cur_version >= self.max_version.unwrap_or(u64::MAX) {
                return Ok(());
            }
            cur_version
        };
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
        let mut database = RocksDBWithMerkleTreeSubstateStore::standard(self.database_dir.clone());
        let kernel_trace = self.kernel_trace;
        let execution_trace = self.execution_trace;
        let cost_breakdown = self.cost_breakdown;
        let txn_write_thread_handle = thread::spawn(move || {
            let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
            let iter = rx.iter();
            for tx_payload in iter {
                let prepared = prepare_ledger_transaction(&tx_payload);
                let state_updates = execute_ledger_transaction(
                    &database,
                    &scrypto_vm,
                    &network,
                    &prepared,
                    kernel_trace,
                    execution_trace,
                    cost_breakdown,
                )
                .into_state_updates();
                let database_updates =
                    state_updates.create_database_updates::<SpreadPrefixKeyMapper>();
                database.commit(&database_updates);

                let new_state_root_hash = database.get_current_root_hash();
                let new_version = database.get_current_version();

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
