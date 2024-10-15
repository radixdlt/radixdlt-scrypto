use clap::Parser;
use radix_common::prelude::*;

use crate::resim::*;

/// Export the definition of a package
#[derive(Parser, Debug)]
pub struct ExportPackageDefinition {
    /// The package ID
    pub package_address: SimulatorPackageAddress,

    /// The output file
    pub output: PathBuf,

    /// Turn on tracing.
    #[clap(short, long)]
    pub trace: bool,
}

impl ExportPackageDefinition {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), String> {
        match export_package_schema(self.package_address.0) {
            Ok(schema) => {
                write_ensuring_folder_exists(
                    &self.output,
                    scrypto_encode(&schema).map_err(Error::SborEncodeError)?,
                )
                .map_err(Error::IOError)?;
                writeln!(
                    out,
                    "Package definition exported to {}",
                    self.output.to_str().unwrap()
                )
                .map_err(Error::IOError)?;
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }
}
