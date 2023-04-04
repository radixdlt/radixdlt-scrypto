use clap::Parser;
use colored::*;
use radix_engine::types::*;
use radix_engine_common::types::NodeId;
use radix_engine_interface::blueprints::package::PackageCodeSubstate;
use radix_engine_interface::blueprints::package::PackageInfoSubstate;
use radix_engine_stores::interface::CommittableSubstateDatabase;
use radix_engine_stores::interface::StateUpdate;
use radix_engine_stores::interface::StateUpdates;
use std::collections::BTreeSet;
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
        let (code_path, schema_path) = if self.path.extension() != Some(OsStr::new("wasm")) {
            build_package(&self.path, false, false).map_err(Error::BuildError)?
        } else {
            let code_path = self.path.clone();
            let schema_path = code_path.with_extension("schema");
            (code_path, schema_path)
        };

        let code = fs::read(code_path).map_err(Error::IOError)?;
        let schema = scrypto_decode(
            &fs::read(&schema_path).map_err(|err| Error::IOErrorAtPath(err, schema_path))?,
        )
        .map_err(Error::SborDecodeError)?;

        if let Some(package_address) = self.package_address.clone() {
            let scrypto_interpreter = ScryptoInterpreter::<DefaultWasmEngine>::default();
            let mut substate_db = RocksdbSubstateStore::standard(get_data_dir()?);
            bootstrap(&mut substate_db, &scrypto_interpreter);

            let node_id: NodeId = package_address.0.into();
            let module_id: ModuleId = TypedModuleId::ObjectState.into();
            let substate_key_code: SubstateKey = PackageOffset::Code.into();
            let package_code = PackageCodeSubstate { code };

            let substate_key_info: SubstateKey = PackageOffset::Info.into();
            let package_info = PackageInfoSubstate {
                schema,
                dependent_resources: BTreeSet::new(),
                dependent_components: BTreeSet::new(),
            };
            let state_updates = StateUpdates {
                substate_changes: indexmap!(
                    (node_id, module_id, substate_key_code) => StateUpdate::Upsert(
                        scrypto_encode(&package_code).unwrap(),
                        None,
                    ),
                    (node_id, module_id, substate_key_info) => StateUpdate::Upsert(
                        scrypto_encode(&package_info).unwrap(),
                        None
                    )
                ),
            };

            substate_db
                .commit(&state_updates)
                .expect("Database misconfigured");

            writeln!(out, "Package updated!").map_err(Error::IOError)?;
        } else {
            let owner_badge_non_fungible_global_id = self
                .owner_badge
                .clone()
                .map(|owner_badge| owner_badge.0)
                .unwrap_or(get_default_owner_badge()?);

            let manifest = ManifestBuilder::new()
                .lock_fee(FAUCET_COMPONENT, 100u32.into())
                .publish_package_with_owner(code, schema, owner_badge_non_fungible_global_id)
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
                    receipt.expect_commit(true).new_package_addresses()[0]
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
