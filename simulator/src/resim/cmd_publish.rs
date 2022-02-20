use clap::Parser;
use colored::*;
use radix_engine::transaction::*;
use scrypto::engine::types::*;
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

    /// The package ID, for overwriting
    #[clap(long)]
    package_id: Option<PackageId>,

    /// The transaction signers
    #[clap(short, long)]
    signers: Option<Vec<EcdsaPublicKey>>,

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

        let mut ledger = RadixEngineDB::with_bootstrap(get_data_dir()?);
        let mut executor = TransactionExecutor::new(&mut ledger, self.trace);
        if let Some(package_id) = self.package_id.clone() {
            // Overwrite package
            executor.overwrite_package(package_id, &code);
            println!("Package updated!");
            Ok(())
        } else {
            match executor.publish_package(&code) {
                Ok(package_id) => {
                    println!("Success! New Package: {}", package_id.to_string().green());
                    Ok(())
                }
                Err(error) => Err(Error::TransactionExecutionError(error)),
            }
        }
    }
}
