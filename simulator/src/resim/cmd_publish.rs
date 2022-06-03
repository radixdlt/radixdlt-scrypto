use clap::Parser;
use colored::*;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use transaction::builder::ManifestBuilder;

use crate::resim::*;
use crate::utils::*;

/// Publish a package
#[derive(Parser, Debug)]
pub struct Publish {
    /// the path to a Scrypto package or a .wasm file
    path: PathBuf,

    /// Output a transaction manifest without execution
    #[clap(short, long)]
    manifest: Option<PathBuf>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl Publish {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        // Load wasm code
        let code = fs::read(if self.path.extension() != Some(OsStr::new("wasm")) {
            build_package(&self.path, false).map_err(Error::CargoError)?
        } else {
            self.path.clone()
        })
        .map_err(Error::IOError)?;

        let manifest = ManifestBuilder::new()
            .publish_package(extract_package(code).map_err(Error::PackageError)?)
            .build();

        let receipt = handle_manifest(manifest, &None, &self.manifest, self.trace, false, out)?;
        if let Some(receipt) = receipt {
            writeln!(
                out,
                "Success! New Package: {}",
                receipt.new_package_addresses[0].to_string().green()
            )
            .map_err(Error::IOError)?;
        }
        Ok(())
    }
}
