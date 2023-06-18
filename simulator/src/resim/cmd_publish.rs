use clap::Parser;
use colored::*;
use radix_engine::types::*;
use radix_engine_common::types::NodeId;
use radix_engine_interface::blueprints::package::{
    BlueprintDefinition, BlueprintDependencies, BlueprintType, FunctionSchema, IndexedStateSchema,
    PackageExport, TypePointer, VmType, PACKAGE_BLUEPRINTS_PARTITION_OFFSET,
    PACKAGE_BLUEPRINT_DEPENDENCIES_PARTITION_OFFSET, PACKAGE_SCHEMAS_PARTITION_OFFSET,
};
use radix_engine_interface::blueprints::package::{PackageCodeSubstate, PackageDefinition};
use radix_engine_interface::schema::TypeRef;
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
        let package_definition: PackageDefinition = manifest_decode(
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
            let package_code = PackageCodeSubstate {
                vm_type: VmType::ScryptoV1,
                code,
            };

            let blueprints_partition_key = SpreadPrefixKeyMapper::to_db_partition_key(
                &node_id,
                MAIN_BASE_PARTITION
                    .at_offset(PACKAGE_BLUEPRINTS_PARTITION_OFFSET)
                    .unwrap(),
            );
            let schemas_partition_key = SpreadPrefixKeyMapper::to_db_partition_key(
                &node_id,
                MAIN_BASE_PARTITION
                    .at_offset(PACKAGE_SCHEMAS_PARTITION_OFFSET)
                    .unwrap(),
            );
            let dependencies_partition_key = SpreadPrefixKeyMapper::to_db_partition_key(
                &node_id,
                MAIN_BASE_PARTITION
                    .at_offset(PACKAGE_BLUEPRINT_DEPENDENCIES_PARTITION_OFFSET)
                    .unwrap(),
            );
            let mut blueprint_updates = index_map_new();
            let mut dependency_updates = index_map_new();
            let mut schema_updates = index_map_new();

            let code_hash = hash(scrypto_encode(&package_code).unwrap());

            for (b, s) in package_definition.blueprints {
                let mut functions = BTreeMap::new();
                let mut function_exports = BTreeMap::new();

                let blueprint_schema = s.schema.clone();
                let schema_hash = hash(scrypto_encode(&blueprint_schema).unwrap());
                let key = SpreadPrefixKeyMapper::map_to_db_sort_key(
                    &scrypto_encode(&schema_hash).unwrap(),
                );
                let update = DatabaseUpdate::Set(scrypto_encode(&blueprint_schema).unwrap());
                schema_updates.insert(key, update);

                for (function, setup) in s.schema.functions.functions {
                    functions.insert(
                        function.clone(),
                        FunctionSchema {
                            receiver: setup.receiver,
                            input: match setup.input {
                                TypeRef::Static(type_index) => {
                                    TypePointer::Package(schema_hash, type_index)
                                }
                                TypeRef::Generic(index) => TypePointer::Instance(index),
                            },
                            output: match setup.output {
                                TypeRef::Static(type_index) => {
                                    TypePointer::Package(schema_hash, type_index)
                                }
                                TypeRef::Generic(index) => TypePointer::Instance(index),
                            },
                        },
                    );
                    let export = PackageExport {
                        code_hash,
                        export_name: setup.export.clone(),
                    };
                    function_exports.insert(function, export);
                }

                let events = s
                    .schema
                    .events
                    .event_schema
                    .into_iter()
                    .map(|(key, index)| {
                        (
                            key,
                            match index {
                                TypeRef::Static(index) => TypePointer::Package(schema_hash, index),
                                TypeRef::Generic(index) => TypePointer::Instance(index),
                            },
                        )
                    })
                    .collect();

                let (feature_set, outer_blueprint) = match s.blueprint_type {
                    BlueprintType::Outer { feature_set } => (feature_set, None),
                    BlueprintType::Inner { outer_blueprint } => {
                        (BTreeSet::new(), Some(outer_blueprint))
                    }
                };

                let def = BlueprintDefinition {
                    interface: BlueprintInterface {
                        generics: s.schema.generics,
                        outer_blueprint,
                        feature_set,
                        functions,
                        events,
                        state: IndexedStateSchema::from_schema(schema_hash, s.schema.state),
                    },
                    function_exports,
                    virtual_lazy_load_functions: s
                        .schema
                        .functions
                        .virtual_lazy_load_functions
                        .into_iter()
                        .map(|(key, export_name)| {
                            (
                                key,
                                PackageExport {
                                    code_hash,
                                    export_name,
                                },
                            )
                        })
                        .collect(),
                };
                let key = SpreadPrefixKeyMapper::map_to_db_sort_key(&scrypto_encode(&b).unwrap());
                let update = DatabaseUpdate::Set(scrypto_encode(&def).unwrap());
                blueprint_updates.insert(key, update);

                let config = BlueprintDependencies {
                    dependencies: s.dependencies,
                };
                let key = SpreadPrefixKeyMapper::map_to_db_sort_key(&scrypto_encode(&b).unwrap());
                let update = DatabaseUpdate::Set(
                    scrypto_encode(&KeyValueEntrySubstate::entry(config)).unwrap(),
                );
                dependency_updates.insert(key, update);
            }

            let database_updates = indexmap!(
                blueprints_partition_key => blueprint_updates,
                dependencies_partition_key => dependency_updates,
                schemas_partition_key => schema_updates,
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
