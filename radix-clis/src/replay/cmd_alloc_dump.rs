use super::ledger_transaction_execution::*;
use super::txn_reader::TxnReader;
use super::Error;
use clap::Parser;
use flate2::read::GzDecoder;
use flume;
use radix_common::prelude::*;
use radix_engine::vm::VmModules;
use radix_engine_profiling::info_alloc::*;
use radix_substate_store_impls::rocks_db_with_merkle_tree::RocksDBWithMerkleTreeSubstateStore;
use radix_substate_store_interface::interface::*;
use radix_transactions::prelude::*;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tar::Archive;

/// Run transactions in archive using RocksDB and dump memory allocations
#[derive(Parser, Debug)]
pub struct TxnAllocDump {
    /// Path to the source Node state manager database
    pub source: PathBuf,
    /// Path to a folder for storing state
    pub database_dir: PathBuf,
    /// Path to the output file
    pub output_file: PathBuf,

    /// The network to use, [mainnet | stokenet]
    #[clap(short, long)]
    pub network: Option<String>,
    /// The max version to execute
    #[clap(short, long)]
    pub max_version: Option<u64>,

    /// Exclude user type of transactions from output data
    #[clap(long)]
    pub exclude_user_transaction: bool,
    /// Include genesis type of transactions in output data
    #[clap(short = 'g', long)]
    pub include_generic_transaction: bool,
    /// Include round update type of transactions in output data
    #[clap(short = 'r', long)]
    pub include_round_update_transaction: bool,

    /// Trace transaction execution
    #[clap(long)]
    pub trace: bool,
}

impl TxnAllocDump {
    pub fn run(&self) -> Result<(), String> {
        let network = match &self.network {
            Some(n) => NetworkDefinition::from_str(n).map_err(Error::ParseNetworkError)?,
            None => NetworkDefinition::mainnet(),
        };
        let address_encoder = TransactionHashBech32Encoder::new(&network);

        let cur_version = {
            let database = RocksDBWithMerkleTreeSubstateStore::standard(self.database_dir.clone());
            let cur_version = database.get_current_version();
            if cur_version >= self.max_version.unwrap_or(u64::MAX) {
                return Ok(());
            }
            cur_version
        };
        let to_version = self.max_version.clone();

        if self.exclude_user_transaction
            && !self.include_generic_transaction
            && !self.include_round_update_transaction
        {
            println!("Nothing selected to dump.");
            return Ok(());
        }

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
            return Err(Error::InvalidTransactionSource.into());
        };
        let txn_read_thread_handle =
            thread::spawn(move || txn_reader.read(cur_version, to_version, tx));

        // txn executor
        let mut database = RocksDBWithMerkleTreeSubstateStore::standard(self.database_dir.clone());
        let exists = self.output_file.exists();
        let mut output = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(&self.output_file)
            .map_err(Error::IOError)?;
        if !exists {
            writeln!(
                output,
                "TXType,TXID,Execution Cost Units,Heap allocations sum,Heap current level,Heap peak memory",
            )
            .map_err(Error::IOError)?;
        }

        let (dump_user, dump_genesis, dump_round) = (
            !self.exclude_user_transaction,
            self.include_generic_transaction,
            self.include_round_update_transaction,
        );
        let trace = self.trace;
        let txn_write_thread_handle = thread::spawn(move || {
            let vm_modules = VmModules::default();
            let iter = rx.iter();
            for tx_payload in iter {
                INFO_ALLOC.set_enable(true);
                INFO_ALLOC.reset_counters();

                let (kinded_hash, receipt) = execute_ledger_transaction(
                    &database,
                    &vm_modules,
                    &network,
                    &tx_payload,
                    trace,
                );

                let (heap_allocations_sum, heap_current_level, heap_peak_memory) =
                    INFO_ALLOC.get_counters_value();
                INFO_ALLOC.set_enable(false);

                let execution_cost_units = receipt
                    .fee_summary()
                    .map(|x| x.total_execution_cost_units_consumed.clone());
                let database_updates = receipt.into_state_updates().create_database_updates();
                database.commit(&database_updates);
                match kinded_hash {
                    LedgerTransactionKindedHash::User(hash) => {
                        if dump_user {
                            writeln!(
                                output,
                                "user,{},{},{},{},{}",
                                address_encoder.encode(&hash).unwrap(),
                                execution_cost_units.unwrap(),
                                heap_allocations_sum,
                                heap_current_level,
                                heap_peak_memory
                            )
                            .map_err(Error::IOError)?
                        }
                    }
                    LedgerTransactionKindedHash::Genesis(hash) => {
                        if dump_genesis {
                            writeln!(
                                output,
                                "genesis,{},{},{},{},{}",
                                hash.0,
                                execution_cost_units.unwrap_or_default(),
                                heap_allocations_sum,
                                heap_current_level,
                                heap_peak_memory
                            )
                            .map_err(Error::IOError)?
                        }
                    }
                    LedgerTransactionKindedHash::Validator(hash) => {
                        if dump_round {
                            writeln!(
                                output,
                                "validator,{},{},{},{},{}",
                                hash,
                                execution_cost_units.unwrap_or_default(),
                                heap_allocations_sum,
                                heap_current_level,
                                heap_peak_memory
                            )
                            .map_err(Error::IOError)?
                        }
                    }
                    LedgerTransactionKindedHash::ProtocolUpdate(hash) => {
                        if dump_round {
                            writeln!(
                                output,
                                "protocol_update,{},{},{},{},{}",
                                hash,
                                execution_cost_units.unwrap_or_default(),
                                heap_allocations_sum,
                                heap_current_level,
                                heap_peak_memory
                            )
                            .map_err(Error::IOError)?
                        }
                    }
                }

                let new_version = database.get_current_version();

                if new_version < 1000 || new_version % 1000 == 0 {
                    let new_state_root_hash = database.get_current_root_hash();
                    print_progress(start.elapsed(), new_version, new_state_root_hash);
                }
            }

            let duration = start.elapsed();
            println!("Time elapsed: {:?}", duration);
            println!("State version: {}", database.get_current_version());
            println!("State root hash: {}", database.get_current_root_hash());
            Ok::<(), Error>(())
        });

        txn_read_thread_handle.join().unwrap()?;
        txn_write_thread_handle.join().unwrap()?;

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
