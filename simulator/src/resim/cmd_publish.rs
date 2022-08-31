use clap::Parser;
use colored::*;
use radix_engine::engine::Substate;
use radix_engine::ledger::{OutputValue, ReadableSubstateStore, WriteableSubstateStore};
use scrypto::core::NetworkDefinition;
use scrypto::engine::types::SubstateId;
use scrypto::prelude::SYS_FAUCET_COMPONENT;
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

    /// The package ID, for overwriting
    #[clap(long)]
    package_address: Option<PackageAddress>,

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

        let package = extract_package(code).map_err(Error::ExtractAbiError)?;

        if let Some(package_address) = self.package_address.clone() {
            let substate_id = SubstateId::Package(package_address);

            let mut substate_store = RadixEngineDB::with_bootstrap(get_data_dir()?);

            let previous_version = substate_store
                .get_substate(&substate_id)
                .map(|output| output.version);

            let validated_package = ValidatedPackage::new(package).map_err(Error::PrepareError)?;
            let output_value = OutputValue {
                substate: Substate::Package(validated_package),
                version: previous_version.unwrap_or(0),
            };

            // Overwrite package
            // TODO: implement real package overwrite
            substate_store.put_substate(SubstateId::Package(package_address), output_value);
            writeln!(out, "Package updated!").map_err(Error::IOError)?;
        } else {
            let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
                .lock_fee(100.into(), SYS_FAUCET_COMPONENT)
                .publish_package(package)
                .build();

            let receipt = handle_manifest(
                manifest,
                &None,
                &self.manifest,
                false,
                self.trace,
                false,
                out,
            )?;
            if let Some(receipt) = receipt {
                writeln!(
                    out,
                    "Success! New Package: {}",
                    receipt.expect_commit().entity_changes.new_package_addresses[0]
                        .to_string()
                        .green()
                )
                .map_err(Error::IOError)?;
            }
        }

        Ok(())
    }
}
