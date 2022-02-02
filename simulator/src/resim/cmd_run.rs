use clap::Parser;
use radix_engine::model::*;
use scrypto::types::*;

use crate::resim::*;
use std::path::PathBuf;

/// Compile and run a transaction manifest
#[derive(Parser, Debug)]
pub struct Run {
    /// the path to a transaction manifest file
    path: PathBuf,

    /// The transaction signers
    #[clap(short, long)]
    signers: Option<Vec<Address>>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl Run {
    pub fn run(&self) -> Result<(), Error> {
        let mut runner = TransactionRunner::new()?;
        let default_signers = runner.default_signers()?;
        let manifest = std::fs::read_to_string(&self.path).map_err(Error::IOError)?;
        let mut transaction =
            transaction_manifest::compile(&manifest).map_err(Error::CompileError)?;
        transaction.instructions.push(Instruction::End {
            signatures: self.signers.clone().unwrap_or(default_signers),
        });
        runner.run_transaction(transaction, self.trace, |receipt| println!("{:?}", receipt))
    }
}
