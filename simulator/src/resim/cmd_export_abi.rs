use clap::Parser;
use radix_engine::transaction::*;
use scrypto::engine::types::*;

use crate::resim::*;

/// Export the ABI of a blueprint
#[derive(Parser, Debug)]
pub struct ExportAbi {
    /// The package ID
    package_address: PackageAddress,

    /// The blueprint name
    blueprint_name: String,

    /// Turn on tracing.
    #[clap(short, long)]
    trace: bool,
}

impl ExportAbi {
    pub fn run(&self) -> Result<(), Error> {
        let mut ledger = RadixEngineDB::with_bootstrap(get_data_dir()?);
        let executor = TransactionExecutor::new(&mut ledger, self.trace);
        match executor.export_abi(self.package_address, &self.blueprint_name) {
            Ok(a) => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&a).map_err(Error::JSONError)?
                );
                Ok(())
            }
            Err(e) => Err(Error::AbiExportError(e)),
        }
    }
}
