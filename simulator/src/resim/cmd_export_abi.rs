use clap::Parser;
use radix_engine::types::*;

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
        let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::local_simulator());

        match export_abi(self.package_address, &self.blueprint_name) {
            Ok(a) => {
                let blueprint = abi::Blueprint {
                    package_address: bech32_encoder
                        .encode_package_address(&self.package_address)
                        .map_err(Error::AddressError)?,
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
