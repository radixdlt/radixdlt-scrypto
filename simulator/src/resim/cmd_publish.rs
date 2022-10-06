use clap::Parser;
use colored::*;
use radix_engine::ledger::{OutputValue, ReadableSubstateStore, WriteableSubstateStore};
use radix_engine::model::Substate;
use radix_engine::types::*;
use scrypto::prelude::ContextualDisplay;
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
    package_address: Option<SimulatorPackageAddress>,

    /// The network to use when outputting manifest, [simulator | adapanet | nebunet | mainnet]
    #[clap(short, long)]
    network: Option<String>,

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
        let code_path = if self.path.extension() != Some(OsStr::new("wasm")) {
            build_package(&self.path, false).map_err(Error::BuildError)?
        } else {
            self.path.clone()
        };
        let abi_path = code_path.with_extension("abi");

        let code = fs::read(&code_path).map_err(Error::IOError)?;
        let abi = scrypto_decode(&fs::read(&abi_path).map_err(Error::IOError)?)
            .map_err(Error::DataError)?;

        if let Some(package_address) = self.package_address.clone() {
            let substate_id = SubstateId::Package(package_address.0);

            let mut substate_store = RadixEngineDB::with_bootstrap(get_data_dir()?);

            let previous_version = substate_store
                .get_substate(&substate_id)
                .map(|output| output.version);

            let validated_package = PackageSubstate {
                code,
                blueprint_abis: abi,
            };
            let output_value = OutputValue {
                substate: Substate::Package(validated_package),
                version: previous_version.unwrap_or(0),
            };

            // Overwrite package
            // TODO: implement real package overwrite
            substate_store.put_substate(SubstateId::Package(package_address.0), output_value);
            writeln!(out, "Package updated!").map_err(Error::IOError)?;
        } else {
            let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
                .lock_fee(100.into(), SYS_FAUCET_COMPONENT)
                .publish_package(code, abi)
                .build();

            let receipt = handle_manifest(
                manifest,
                &None,
                &self.network,
                &self.manifest,
                self.trace,
                false,
                out,
            )?;
            if let Some(receipt) = receipt {
                writeln!(
                    out,
                    "Success! New Package: {}",
                    receipt.expect_commit().entity_changes.new_package_addresses[0]
                        .display(&Bech32Encoder::for_simulator())
                        .to_string()
                        .green()
                )
                .map_err(Error::IOError)?;
            }
        }

        Ok(())
    }
}
