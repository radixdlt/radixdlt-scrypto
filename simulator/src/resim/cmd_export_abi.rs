use clap::Parser;
use radix_engine::transaction::*;
use scrypto::types::*;

use crate::ledger::*;
use crate::resim::*;

/// Export the ABI of a blueprint
#[derive(Parser, Debug)]
pub struct ExportAbi {
    /// The package address
    package_address: Address,

    /// The blueprint name
    blueprint_name: String,

    /// Turn on tracing.
    #[clap(short, long)]
    trace: bool,
}

impl ExportAbi {
    pub fn run(&self) -> Result<(), Error> {
        let configs = get_configs()?;
        let mut ledger = FileBasedLedger::with_bootstrap(get_data_dir()?);
        let executor = TransactionExecutor::new(
            &mut ledger,
            configs.current_epoch,
            configs.nonce,
            self.trace,
        );
        let abi = executor.export_abi(self.package_address, &self.blueprint_name);

        match abi {
            Err(e) => Err(Error::TransactionExecutionError(e)),
            Ok(a) => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&a).map_err(Error::JSONError)?
                );
                Ok(())
            }
        }
    }
}
