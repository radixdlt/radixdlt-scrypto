use super::Error;
use clap::Parser;
use radix_engine::types::*;
use std::path::PathBuf;

/// Run transactions in archive using RocksDB and dump memory allocations
#[derive(Parser, Debug)]
pub struct TxnAllocDump {
    /// Path to the source Node state manager database
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

impl TxnAllocDump {
    pub fn run(&self) -> Result<(), Error> {
        Ok(())
    }
}
