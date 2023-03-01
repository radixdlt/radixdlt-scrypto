use clap::Parser;
use radix_engine::types::*;

use crate::resim::*;

/// Export the ABI of a blueprint
#[derive(Parser, Debug)]
pub struct ExportPackageSchema {
    /// The package ID
    pub package_address: SimulatorPackageAddress,

    /// Turn on tracing.
    #[clap(short, long)]
    pub trace: bool,
}

impl ExportPackageSchema {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        match export_package_schema(self.package_address.0) {
            Ok(schema) => {
                writeln!(
                    out,
                    "{}",
                    serde_json::to_string_pretty(&schema).map_err(Error::JSONError)?
                )
                .map_err(Error::IOError)?;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
