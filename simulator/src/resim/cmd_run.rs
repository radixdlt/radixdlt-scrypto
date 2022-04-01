use clap::Parser;
use radix_engine::model::*;
use std::path::PathBuf;

use crate::resim::*;

/// Compile and run a transaction manifest
#[derive(Parser, Debug)]
pub struct Run {
    /// the path to a transaction manifest file
    path: PathBuf,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl Run {
    pub fn run(&self) -> Result<(), Error> {
        let mut ledger = RadixEngineDB::with_bootstrap(get_data_dir()?);
        let mut executor = TransactionExecutor::new(&mut ledger, self.trace);
        let (_, default_sks) = get_default_signers()?;
        let manifest = std::fs::read_to_string(&self.path).map_err(Error::IOError)?;
        let mut transaction =
            transaction_manifest::compile(&manifest).map_err(Error::CompileError)?;

        transaction.instructions.push(Instruction::Nonce {
            nonce: executor.substate_store().get_nonce(),
        });
        transaction = transaction.sign(&default_sks);
        process_transaction(transaction, &mut executor, &None)
    }
}
