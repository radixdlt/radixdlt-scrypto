use clap::Parser;
use radix_engine::model::*;
use radix_engine::transaction::*;
use scrypto::types::*;

use crate::ledger::*;
use crate::resim::*;
use std::path::PathBuf;

/// Compile and run a transaction manifest
#[derive(Parser, Debug)]
pub struct Run {
    /// the path to a transaction manifest file
    path: PathBuf,

    /// The transaction signers
    #[clap(short, long)]
    signers: Vec<Address>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl Run {
    pub fn run(&self) -> Result<(), Error> {
        let manifest = std::fs::read_to_string(&self.path).map_err(Error::IOError)?;
        let mut transaction =
            transaction_manifest::compile(&manifest).map_err(Error::CompileError)?;
        transaction.instructions.push(Instruction::End {
            signatures: self.signers.clone(),
        });

        let mut configs = get_configs()?;
        let mut ledger = FileBasedLedger::with_bootstrap(get_data_dir()?);
        let mut executor = TransactionExecutor::new(
            &mut ledger,
            configs.current_epoch,
            configs.nonce,
            self.trace,
        );
        let receipt = executor
            .run(transaction)
            .map_err(Error::TransactionValidationError)?;

        println!("{:?}", receipt);
        if receipt.result.is_ok() {
            configs.nonce = executor.nonce();
            set_configs(configs)?;
        }

        receipt.result.map_err(Error::TransactionExecutionError)
    }
}
