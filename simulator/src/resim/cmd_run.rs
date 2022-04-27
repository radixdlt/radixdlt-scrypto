use clap::Parser;
use std::path::PathBuf;

use crate::resim::*;

/// Compiles, signs and runs a transaction manifest
#[derive(Parser, Debug)]
pub struct Run {
    /// The path to a transaction manifest file
    path: PathBuf,

    /// The private keys used for signing, separated by comma
    #[clap(short, long)]
    signing_keys: Option<String>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl Run {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let mut ledger = RadixEngineDB::with_bootstrap(get_data_dir()?);
        let mut executor = TransactionExecutor::new(&mut ledger, self.trace);
        let manifest = std::fs::read_to_string(&self.path).map_err(Error::IOError)?;
        let transaction = transaction_manifest::compile(&manifest).map_err(Error::CompileError)?;
        process_transaction(&mut executor, transaction, &self.signing_keys, &None, out)
    }
}
