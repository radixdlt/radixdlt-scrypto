use clap::Parser;
use radix_engine::transaction::*;
use radix_engine::wasm::*;
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
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let mut substate_store = RadixEngineDB::with_bootstrap(get_data_dir()?);
        let mut wasm_engine = default_wasm_engine();
        let executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, self.trace);
        match executor.export_abi(self.package_address, &self.blueprint_name) {
            Ok(a) => {
                writeln!(
                    out,
                    "{}",
                    serde_json::to_string_pretty(&a).map_err(Error::JSONError)?
                )
                .map_err(Error::IOError)?;
                Ok(())
            }
            Err(e) => Err(Error::AbiExportError(e)),
        }
    }
}
