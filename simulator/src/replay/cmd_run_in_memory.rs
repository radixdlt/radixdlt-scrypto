use crate::replay::decompress_entry;
use crate::replay::execute_ledger_transaction;
use crate::replay::print_progress;
use crate::replay::Error;
use clap::Parser;
use flate2::read::GzDecoder;
use radix_engine::types::*;
use radix_engine::vm::wasm::*;
use radix_engine::vm::ScryptoVm;
use radix_engine_interface::prelude::NetworkDefinition;
use radix_engine_store_interface::db_key_mapper::SpreadPrefixKeyMapper;
use radix_engine_store_interface::interface::CommittableSubstateDatabase;
use radix_engine_stores::hash_tree_support::HashTreeUpdatingDatabase;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use std::fs::File;
use std::path::PathBuf;
use tar::Archive;

/// Run transactions in memory
#[derive(Parser, Debug)]
pub struct RunInMemory {
    /// The transaction file, in `.tar.gz` format, with entries sorted
    pub transaction_file: PathBuf,

    /// The network to use, [mainnet | stokenet]
    #[clap(short, long)]
    pub network: Option<String>,
    /// The max version to execute
    #[clap(short, long)]
    pub max_version: Option<u64>,
    #[clap(long)]
    pub enable_jmt: bool,
}

impl RunInMemory {
    pub fn run(&self) -> Result<(), Error> {
        let network = match &self.network {
            Some(n) => NetworkDefinition::from_str(n).map_err(Error::ParseNetworkError)?,
            None => NetworkDefinition::mainnet(),
        };

        let tar_gz = File::open(&self.transaction_file).map_err(Error::IOError)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);

        let in_memory = InMemorySubstateDatabase::standard();
        let mut database = HashTreeUpdatingDatabase::new(in_memory, self.enable_jmt);
        let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
        let start = std::time::Instant::now();
        for entry in archive.entries().map_err(Error::IOError)? {
            // check limit
            let version = database.get_current_version();
            if version >= self.max_version.unwrap_or(u64::MAX) {
                break;
            }

            // read the entry
            let entry = entry.map_err(|e| Error::IOError(e))?;
            let (tx_version, tx_payload) =
                decompress_entry(entry).ok_or(Error::InvalidTransactionFile)?;
            if tx_version <= version {
                continue;
            }

            // execute transaction
            let state_updates =
                execute_ledger_transaction(&database, &scrypto_vm, network.clone(), &tx_payload);
            let database_updates = state_updates.create_database_updates::<SpreadPrefixKeyMapper>();
            database.commit(&database_updates);

            // print progress
            let new_version = database.get_current_version();
            if new_version < 1000 || new_version % 1000 == 0 {
                let new_root = database.get_current_root_hash();
                let duration = start.elapsed();
                print_progress(duration, new_version, new_root);
            }
        }
        let duration = start.elapsed();
        println!("Time elapsed: {:?}", duration);
        println!("State version: {}", database.get_current_version());
        println!("State root hash: {}", database.get_current_root_hash());

        Ok(())
    }
}
