use clap::Parser;
use radix_engine::ledger::SubstateStore;
use radix_engine::model::*;
use radix_engine::transaction::*;
use scrypto::types::*;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

use crate::ledger::*;
use crate::resim::*;
use crate::utils::*;

/// Publish a package
#[derive(Parser, Debug)]
pub struct Publish {
    /// the path to a Scrypto package or a .wasm file
    path: PathBuf,

    /// The package address, for overwriting
    #[clap(long)]
    address: Option<Address>,

    /// The transaction signers
    #[clap(short, long)]
    signers: Option<Vec<Address>>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl Publish {
    pub fn run(&self) -> Result<(), Error> {
        // Load wasm code
        let code = fs::read(if self.path.extension() != Some(OsStr::new("wasm")) {
            build_package(&self.path, false).map_err(Error::CargoError)?
        } else {
            self.path.clone()
        })
        .map_err(Error::IOError)?;

        if let Some(address) = self.address.clone() {
            // Overwrite package
            let mut ledger = RadixEngineDB::with_bootstrap(get_data_dir()?);
            ledger.put_package(address, Package::new(code));
            println!("Package updated!");
            Ok(())
        } else {
            let mut ledger = RadixEngineDB::with_bootstrap(get_data_dir()?);
            let mut executor = TransactionExecutor::new(&mut ledger, self.trace);
            let default_signers = get_default_signers()?;
            let transaction = TransactionBuilder::new(&executor)
                .publish_package(&code)
                .build(self.signers.clone().unwrap_or(default_signers))
                .map_err(Error::TransactionConstructionError)?;
            let receipt = executor
                .run(transaction)
                .map_err(Error::TransactionValidationError)?;
            println!("{:?}", receipt);
            receipt.result.map_err(Error::TransactionExecutionError)
        }
    }
}
