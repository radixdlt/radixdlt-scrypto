use clap::Parser;
use radix_engine::model::*;

use crate::resim::*;
use std::path::PathBuf;

/// Compile and run a transaction manifest
#[derive(Parser, Debug)]
pub struct Run {
    /// the path to a transaction manifest file
    path: PathBuf,

    /// The transaction signers
    #[clap(short, long)]
    signers: Option<Vec<PublicKey>>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl Run {
    pub fn run(&self) -> Result<(), Error> {
        let mut ledger = RadixEngineDB::with_bootstrap(get_data_dir()?);
        let mut executor = TransactionExecutor::new(&mut ledger, self.trace);
        let default_signers = get_default_signers()?;
        let manifest = std::fs::read_to_string(&self.path).map_err(Error::IOError)?;
        let mut transaction =
            transaction_manifest::compile(&manifest).map_err(Error::CompileError)?;
        let signatures = self.signers.clone().map(|v| v.into_iter().map(|k| k.0).collect())
            .unwrap_or(default_signers);
        transaction.instructions.push(Instruction::End { signatures });
        process_transaction(transaction, &mut executor, &None)
    }
}
