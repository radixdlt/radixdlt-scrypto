use clap::Parser;
use colored::*;
use radix_engine::types::*;
use radix_engine_common::types::NodeId;
use radix_engine_interface::blueprints::package::{BlueprintDefinition, PACKAGE_BLUEPRINTS_PARTITION_OFFSET};
use radix_engine_interface::blueprints::package::{PackageCodeSubstate, PackageSetup};
use radix_engine_store_interface::{
    db_key_mapper::{DatabaseKeyMapper, SpreadPrefixKeyMapper},
    interface::{CommittableSubstateDatabase, DatabaseUpdate},
};
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
        let (code_path, definition_path) = if self.path.extension() != Some(OsStr::new("wasm")) {
            build_package(&self.path, false, false).map_err(Error::BuildError)?
        } else {
            let code_path = self.path.clone();
            let schema_path = code_path.with_extension("schema");
            (code_path, schema_path)
        };

        let code = fs::read(code_path).map_err(Error::IOError)?;
        let package_definition: PackageSetup = manifest_decode(
            &fs::read(&definition_path)
                .map_err(|err| Error::IOErrorAtPath(err, definition_path))?,
        )
        .map_err(Error::SborDecodeError)?;

        if let Some(package_address) = self.package_address.clone() {
            let scrypto_interpreter = ScryptoVm::<DefaultWasmEngine>::default();
            let mut substate_db = RocksdbSubstateStore::standard(get_data_dir()?);
            Bootstrapper::new(&mut substate_db, &scrypto_interpreter, false)
                .bootstrap_test_default();

            let node_id: NodeId = package_address.0.into();
            let fields_partition_key =
                SpreadPrefixKeyMapper::to_db_partition_key(&node_id, MAIN_BASE_PARTITION);
            let code_db_sort_key =
                SpreadPrefixKeyMapper::to_db_sort_key(&PackageField::Code.into());
            let package_code = PackageCodeSubstate { code };

            let blueprints_partition_key = SpreadPrefixKeyMapper::to_db_partition_key(
                &node_id,
                MAIN_BASE_PARTITION.at_offset(PACKAGE_BLUEPRINTS_PARTITION_OFFSET).unwrap(),
            );
            let mut blueprint_updates = index_map_new();

            for (b, s) in package_definition.blueprints {
                let def = BlueprintDefinition {
                    virtual_lazy_load_functions: s.virtual_lazy_load_functions,
                    schema: s.schema,
                    blueprint: s.blueprint.into(),
                    template: s.template,
                };
                let key = SpreadPrefixKeyMapper::map_to_db_sort_key(&scrypto_encode(&b).unwrap());
                let update = DatabaseUpdate::Set(scrypto_encode(&def).unwrap());
                blueprint_updates.insert(key, update);
            }

            let database_updates = indexmap!(
                fields_partition_key => indexmap!(
                    code_db_sort_key => DatabaseUpdate::Set(
                        scrypto_encode(&package_code).unwrap()
                    ),
                ),
                blueprints_partition_key => blueprint_updates,
            );

            substate_db.commit(&database_updates);

            writeln!(out, "Package updated!").map_err(Error::IOError)?;
        } else {
            let owner_badge_non_fungible_global_id = self
                .owner_badge
                .clone()
                .map(|owner_badge| owner_badge.0)
                .unwrap_or(get_default_owner_badge()?);

            let manifest = ManifestBuilder::new()
                .lock_fee(FAUCET, 100u32.into())
                .publish_package_with_owner(
                    code,
                    package_definition,
                    owner_badge_non_fungible_global_id,
                )
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
