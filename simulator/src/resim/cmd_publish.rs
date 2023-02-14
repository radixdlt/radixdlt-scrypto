use clap::Parser;
use colored::*;
use radix_engine::ledger::{OutputValue, ReadableSubstateStore, WriteableSubstateStore};
use radix_engine::types::*;
use radix_engine_interface::api::types::RENodeId;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use transaction::builder::ManifestBuilder;
use utils::ContextualDisplay;

use crate::resim::*;
use crate::utils::*;

/// Publish a package
#[derive(Parser, Debug)]
pub struct Publish {
    /// the path to a Scrypto package or a .wasm file
    pub path: PathBuf,

    /// The owner badge (hex value).
    #[clap(long)]
    pub owner_badge: Option<SimulatorNonFungibleGlobalId>,

    /// The address of an existing package to overwrite
    #[clap(long)]
    pub package_address: Option<SimulatorPackageAddress>,

    /// The network to use when outputting manifest, [simulator | adapanet | nebunet | mainnet]
    #[clap(short, long)]
    pub network: Option<String>,

    /// Output a transaction manifest without execution
    #[clap(short, long)]
    pub manifest: Option<PathBuf>,

    /// Turn on tracing
    #[clap(short, long)]
    pub trace: bool,
}

impl Publish {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        // Load wasm code
        let (code_path, abi_path) = if self.path.extension() != Some(OsStr::new("wasm")) {
            build_package(&self.path, false, false).map_err(Error::BuildError)?
        } else {
            let code_path = self.path.clone();
            let abi_path = code_path.with_extension("abi");
            (code_path, abi_path)
        };

        let code = fs::read(code_path).map_err(Error::IOError)?;
        let abi = scrypto_decode(
            &fs::read(&abi_path).map_err(|err| Error::IOErrorAtPath(err, abi_path))?,
        )
        .map_err(Error::DataError)?;

        if let Some(package_address) = self.package_address.clone() {
            let scrypto_interpreter = ScryptoInterpreter::<DefaultWasmEngine>::default();
            let mut substate_store =
                RadixEngineDB::with_bootstrap(get_data_dir()?, &scrypto_interpreter);

            let global: GlobalAddressSubstate = substate_store
                .get_substate(&SubstateId(
                    RENodeId::Global(GlobalAddress::Package(package_address.0)),
                    SubstateOffset::Global(GlobalOffset::Global),
                ))
                .map(|s| s.substate)
                .map(|s| s.to_runtime().into())
                .ok_or(Error::PackageAddressNotFound)?;
            let substate_id = SubstateId(
                global.node_deref(),
                SubstateOffset::Package(PackageOffset::Info),
            );

            let previous_version = substate_store
                .get_substate(&substate_id)
                .map(|output| output.version);

            let validated_package = PackageInfoSubstate {
                code,
                blueprint_abis: abi,
            };
            let output_value = OutputValue {
                substate: PersistedSubstate::PackageInfo(validated_package),
                version: previous_version.unwrap_or(0),
            };

            // Overwrite package
            // TODO: implement real package overwrite
            substate_store.put_substate(
                SubstateId(
                    global.node_deref(),
                    SubstateOffset::Package(PackageOffset::Info),
                ),
                output_value,
            );
            writeln!(out, "Package updated!").map_err(Error::IOError)?;
        } else {
            let owner_badge_non_fungible_global_id = self
                .owner_badge
                .clone()
                .map(|owner_badge| owner_badge.0)
                .unwrap_or(get_default_owner_badge()?);

            let manifest = ManifestBuilder::new()
                .lock_fee(FAUCET_COMPONENT, 100u32.into())
                .publish_package_with_owner(code, abi, owner_badge_non_fungible_global_id)
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
