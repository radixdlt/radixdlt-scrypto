use clap::Parser;
use colored::*;
use radix_engine::blueprints::models::*;
use radix_engine::track::IntoDatabaseUpdates;
use radix_engine::types::*;
use radix_engine_interface::blueprints::package::{
    BlueprintDefinition, BlueprintDependencies, BlueprintPayloadDef, FunctionSchema,
    IndexedStateSchema, PackageExport, VmType, *,
};
use radix_engine_queries::typed_substate_layout::*;
use radix_engine_store_interface::interface::CommittableSubstateDatabase;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
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
            let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
            let native_vm = DefaultNativeVm::new();
            let vm = Vm::new(&scrypto_vm, native_vm);
            let mut substate_db = RocksdbSubstateStore::standard(get_data_dir()?);
            Bootstrapper::new(&mut substate_db, vm, false).bootstrap_test_default();

            let node_id: NodeId = package_address.0.into();

            let code_hash = CodeHash::from(hash(&code));
            let instrumented_code = WasmValidator::default()
                .validate(&code, package_definition.blueprints.values())
                .map_err(Error::InvalidPackage)?
                .0;

            let mut package_state_to_set = PackageStateInit {
                royalty: None, // No change
                // To be set later in this function
                blueprint_version_definitions: indexmap!(),
                blueprint_version_dependencies: indexmap!(),
                schemas: indexmap!(),
                blueprint_version_royalty_configs: indexmap!(),
                blueprint_version_auth_configs: indexmap!(),
                // Code set now
                code_vm_type: indexmap! {
                    code_hash.into_key() => PackageCodeVmType {
                        vm_type: VmType::ScryptoV1
                    }.into_locked_substate()
                },
                code_original_code: indexmap! {
                    code_hash.into_key() => PackageCodeOriginalCode {
                        code
                    }.into_locked_substate()
                },
                code_instrumented_code: indexmap! {
                    code_hash.into_key() => PackageCodeInstrumentedCode {
                        instrumented_code
                    }.into_locked_substate()
                },
            };

            for (blueprint_name, blueprint_definition) in package_definition.blueprints {
                let mut functions = BTreeMap::new();
                let mut function_exports = BTreeMap::new();

                let blueprint_version_key = BlueprintVersionKey::new_default(blueprint_name);
                let blueprint_schema = blueprint_definition.schema.clone();
                let schema_hash = blueprint_schema.schema.generate_schema_hash();
                package_state_to_set.schemas.insert(
                    schema_hash.into_key(),
                    blueprint_schema.schema.into_locked_substate(),
                );

                for (function, setup) in blueprint_definition.schema.functions.functions {
                    functions.insert(
                        function.clone(),
                        FunctionSchema {
                            receiver: setup.receiver,
                            input: BlueprintPayloadDef::from_type_ref(setup.input, schema_hash),
                            output: BlueprintPayloadDef::from_type_ref(setup.output, schema_hash),
                        },
                    );
                    let export = PackageExport {
                        code_hash,
                        export_name: setup.export.clone(),
                    };
                    function_exports.insert(function, export);
                }

                let events = blueprint_definition
                    .schema
                    .events
                    .event_schema
                    .into_iter()
                    .map(|(key, type_ref)| {
                        (
                            key,
                            BlueprintPayloadDef::from_type_ref(type_ref, schema_hash),
                        )
                    })
                    .collect();

                let state = IndexedStateSchema::from_schema(
                    schema_hash,
                    blueprint_definition.schema.state,
                    Default::default(),
                );

                let def = BlueprintDefinition {
                    interface: BlueprintInterface {
                        generics: blueprint_definition.schema.generics,
                        blueprint_type: blueprint_definition.blueprint_type,
                        is_transient: false,
                        feature_set: blueprint_definition.feature_set,
                        functions,
                        events,
                        state,
                    },
                    function_exports,
                    hook_exports: BTreeMap::new(),
                };
                package_state_to_set.blueprint_version_definitions.insert(
                    blueprint_version_key.clone().into_key(),
                    def.into_locked_substate(),
                );
                package_state_to_set.blueprint_version_dependencies.insert(
                    blueprint_version_key.clone().into_key(),
                    BlueprintDependencies {
                        dependencies: blueprint_definition.dependencies,
                    }
                    .into_locked_substate(),
                );
                package_state_to_set.blueprint_version_auth_configs.insert(
                    blueprint_version_key.clone().into_key(),
                    blueprint_definition.auth_config.into_locked_substate(),
                );
                package_state_to_set
                    .blueprint_version_royalty_configs
                    .insert(
                        blueprint_version_key.clone().into_key(),
                        blueprint_definition.royalty_config.into_locked_substate(),
                    );
            }

            let database_updates = map_package_state_into_main_partition_node_substate_flash(
                package_state_to_set,
                FeatureChecks::None,
            )
            .unwrap()
            .into_database_updates::<SpreadPrefixKeyMapper>(&node_id);
            substate_db.commit(&database_updates);

            writeln!(out, "Package updated!").map_err(Error::IOError)?;
        } else {
            let owner_badge_non_fungible_global_id = self
                .owner_badge
                .clone()
                .map(|owner_badge| owner_badge.0)
                .unwrap_or(get_default_owner_badge()?);

            let manifest = ManifestBuilder::new()
                .lock_fee_from_faucet()
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
                        .display(&AddressBech32Encoder::for_simulator())
                        .to_string()
                        .green()
                )
                .map_err(Error::IOError)?;
            }
        }

        Ok(())
    }
}
