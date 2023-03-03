use clap::Parser;
use radix_engine::types::*;

use crate::resim::*;

/// Export the schema of a package
#[derive(Parser, Debug)]
pub struct ExportSchema {
    /// The package ID
    pub package_address: SimulatorPackageAddress,

    /// The output file
    pub output: PathBuf,

    /// Turn on tracing.
    #[clap(short, long)]
    pub trace: bool,
}

impl ExportSchema {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        match export_package_schema(self.package_address.0) {
            Ok(schema) => {
                fs::write(
                    &self.output,
                    scrypto_encode(&schema).map_err(Error::SborEncodeError)?,
                )
                .map_err(Error::IOError)?;
                writeln!(
                    out,
                    "Blueprint schema exported to {}",
                    self.output.to_str().unwrap()
                )
                .map_err(Error::IOError)?;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
