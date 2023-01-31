use clap::Parser;
use radix_engine::types::*;
use radix_engine_interface::abi;

use crate::resim::*;

/// Export the ABI of a blueprint
#[derive(Parser, Debug)]
pub struct ExportAbi {
    /// The package ID
    pub package_address: SimulatorPackageAddress,

    /// The blueprint name
    pub blueprint_name: String,

    /// Turn on tracing.
    #[clap(short, long)]
    pub trace: bool,
}

impl ExportAbi {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        match export_abi(self.package_address.0, &self.blueprint_name) {
            Ok(a) => {
                let blueprint = abi::Blueprint {
                    package_address: self.package_address.0.to_hex(),
                    blueprint_name: self.blueprint_name.clone(),
                    abi: a,
                };
                writeln!(
                    out,
                    "{}",
                    serde_json::to_string_pretty(&blueprint).map_err(Error::JSONError)?
                )
                .map_err(Error::IOError)?;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
