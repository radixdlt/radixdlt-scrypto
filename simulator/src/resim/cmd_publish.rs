use clap::Parser;
use radix_engine::ledger::Ledger;
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
    signers: Vec<Address>,

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

        // Update existing package if `--address` is provided
        if let Some(address) = self.address.clone() {
            let mut ledger = FileBasedLedger::with_bootstrap(get_data_dir()?);
            ledger.put_package(address, Package::new(code));
            println!("Package updated!");
            Ok(())
        } else {
            let mut configs = get_configs()?;
            let mut ledger = FileBasedLedger::with_bootstrap(get_data_dir()?);
            let mut executor = TransactionExecutor::new(
                &mut ledger,
                configs.current_epoch,
                configs.nonce,
                self.trace,
            );
            let transaction = TransactionBuilder::new(&executor)
                .publish_package(&code)
                .build(self.signers.clone())
                .map_err(Error::TransactionConstructionError)?;
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
}
