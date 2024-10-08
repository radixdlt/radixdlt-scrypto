use clap::Parser;
use colored::*;
use radix_common::prelude::*;
use radix_engine::blueprints::models::*;
use radix_engine::vm::wasm::ScryptoV1WasmValidator;
use radix_engine::vm::ScryptoVmVersion;
use radix_engine_interface::blueprints::package::{
    BlueprintDefinition, BlueprintDependencies, BlueprintPayloadDef, FunctionSchema,
    IndexedStateSchema, PackageExport, VmType, *,
};
use radix_engine_interface::prelude::*;
use radix_rust::ContextualDisplay;
use radix_substate_store_interface::db_key_mapper::*;
use radix_substate_store_interface::interface::*;
use radix_substate_store_queries::typed_substate_layout::*;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

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

    /// When passed, this argument disables wasm-opt from running on the built wasm.
    #[clap(long)]
    disable_wasm_opt: bool,

    /// The max log level, such as ERROR, WARN, INFO, DEBUG and TRACE.
    /// The default is INFO.
    #[clap(long)]
    log_level: Option<Level>,
}

impl Publish {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), String> {
        // Load wasm code
        let (code_path, definition_path) = if self.path.extension() != Some(OsStr::new("wasm")) {
            let build_artifacts = build_package(
                &self.path,
                self.disable_wasm_opt,
                self.log_level.unwrap_or(Level::default()),
                false,
                &[],
            )
            .map_err(Error::BuildError)?;
            if build_artifacts.len() > 1 {
                return Err(Error::BuildError(BuildError::WorkspaceNotSupported).into());
            } else {
                build_artifacts
                    .first()
                    .ok_or(Error::BuildError(BuildError::BuildArtifactsEmpty))?
                    .to_owned()
            }
        } else {
            let code_path = self.path.clone();
            let schema_path = code_path.with_extension("rpd");
            (code_path, schema_path)
        };

        let code = fs::read(code_path).map_err(Error::IOError)?;
        let package_definition: PackageDefinition = manifest_decode(
            &fs::read(&definition_path)
                .map_err(|err| Error::IOErrorAtPath(err, definition_path))?,
        )
        .map_err(Error::SborDecodeError)?;

        if let Some(package_address) = self.package_address.clone() {
            let SimulatorEnvironment { mut db, .. } = SimulatorEnvironment::new()?;

            let node_id: NodeId = package_address.0.into();

            let code_hash = CodeHash::from(hash(&code));
            let blueprints_partition_key = SpreadPrefixKeyMapper::to_db_partition_key(
                &node_id,
                MAIN_BASE_PARTITION
                    .at_offset(PACKAGE_BLUEPRINTS_PARTITION_OFFSET)
                    .unwrap(),
            );
            let schemas_partition_key =
                SpreadPrefixKeyMapper::to_db_partition_key(&node_id, SCHEMAS_PARTITION);
            let dependencies_partition_key = SpreadPrefixKeyMapper::to_db_partition_key(
                &node_id,
                MAIN_BASE_PARTITION
                    .at_offset(PACKAGE_BLUEPRINT_DEPENDENCIES_PARTITION_OFFSET)
                    .unwrap(),
            );
            let royalty_configs_partition_key = SpreadPrefixKeyMapper::to_db_partition_key(
                &node_id,
                MAIN_BASE_PARTITION
                    .at_offset(PACKAGE_ROYALTY_PARTITION_OFFSET)
                    .unwrap(),
            );
            let auth_configs_partition_key = SpreadPrefixKeyMapper::to_db_partition_key(
                &node_id,
                MAIN_BASE_PARTITION
                    .at_offset(PACKAGE_AUTH_TEMPLATE_PARTITION_OFFSET)
                    .unwrap(),
            );
            let vm_type_partition_key = SpreadPrefixKeyMapper::to_db_partition_key(
                &node_id,
                MAIN_BASE_PARTITION
                    .at_offset(PACKAGE_VM_TYPE_PARTITION_OFFSET)
                    .unwrap(),
            );
            let original_code_partition_key = SpreadPrefixKeyMapper::to_db_partition_key(
                &node_id,
                MAIN_BASE_PARTITION
                    .at_offset(PACKAGE_ORIGINAL_CODE_PARTITION_OFFSET)
                    .unwrap(),
            );
            let instrumented_code_partition_key = SpreadPrefixKeyMapper::to_db_partition_key(
                &node_id,
                MAIN_BASE_PARTITION
                    .at_offset(PACKAGE_INSTRUMENTED_CODE_PARTITION_OFFSET)
                    .unwrap(),
            );
            let mut blueprint_updates = index_map_new();
            let mut dependency_updates = index_map_new();
            let mut auth_config_updates = index_map_new();
            let mut royalty_config_updates = index_map_new();
            let mut schema_updates = index_map_new();
            let mut vm_type_updates = index_map_new();
            let mut original_code_updates = index_map_new();
            let mut instrumented_code_updates = index_map_new();
            let instrumented_code = ScryptoV1WasmValidator::new(ScryptoVmVersion::latest())
                .validate(&code, package_definition.blueprints.values())
                .map_err(Error::InvalidPackage)?
                .0;

            let vm_type = PackageCodeVmType {
                vm_type: VmType::ScryptoV1,
            };
            let original_code = PackageCodeOriginalCode { code };
            let instrumented_code = PackageCodeInstrumentedCode { instrumented_code };
            {
                let key =
                    SpreadPrefixKeyMapper::map_to_db_sort_key(&scrypto_encode(&code_hash).unwrap());
                let update =
                    DatabaseUpdate::Set(scrypto_encode(&vm_type.into_locked_substate()).unwrap());
                vm_type_updates.insert(key, update);

                let key =
                    SpreadPrefixKeyMapper::map_to_db_sort_key(&scrypto_encode(&code_hash).unwrap());
                let update = DatabaseUpdate::Set(
                    scrypto_encode(&original_code.into_locked_substate()).unwrap(),
                );
                original_code_updates.insert(key, update);

                let key =
                    SpreadPrefixKeyMapper::map_to_db_sort_key(&scrypto_encode(&code_hash).unwrap());
                let update = DatabaseUpdate::Set(
                    scrypto_encode(&instrumented_code.into_locked_substate()).unwrap(),
                );
                instrumented_code_updates.insert(key, update);
            }

            for (blueprint_name, blueprint_definition) in package_definition.blueprints {
                let mut functions = index_map_new();
                let mut function_exports = index_map_new();

                let blueprint_version_key = BlueprintVersionKey::new_default(blueprint_name);
                let blueprint_schema = blueprint_definition.schema.clone();
                let schema_hash = blueprint_schema.schema.generate_schema_hash();
                schema_updates.insert(
                    SpreadPrefixKeyMapper::map_to_db_sort_key(
                        &scrypto_encode(&schema_hash).unwrap(),
                    ),
                    DatabaseUpdate::Set(
                        scrypto_encode(&blueprint_schema.schema.into_locked_substate()).unwrap(),
                    ),
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

                let types = blueprint_definition
                    .schema
                    .types
                    .type_schema
                    .into_iter()
                    .map(|(key, local_type_id)| {
                        (
                            key,
                            ScopedTypeId(
                                blueprint_definition.schema.schema.generate_schema_hash(),
                                local_type_id,
                            ),
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
                        types,
                    },
                    function_exports,
                    hook_exports: index_map_new(),
                };
                blueprint_updates.insert(
                    SpreadPrefixKeyMapper::map_to_db_sort_key(
                        &scrypto_encode(&blueprint_version_key.clone()).unwrap(),
                    ),
                    DatabaseUpdate::Set(scrypto_encode(&def.into_locked_substate()).unwrap()),
                );
                dependency_updates.insert(
                    SpreadPrefixKeyMapper::map_to_db_sort_key(
                        &scrypto_encode(&blueprint_version_key.clone()).unwrap(),
                    ),
                    DatabaseUpdate::Set(
                        scrypto_encode(
                            &BlueprintDependencies {
                                dependencies: blueprint_definition.dependencies,
                            }
                            .into_locked_substate(),
                        )
                        .unwrap(),
                    ),
                );
                auth_config_updates.insert(
                    SpreadPrefixKeyMapper::map_to_db_sort_key(
                        &scrypto_encode(&blueprint_version_key.clone()).unwrap(),
                    ),
                    DatabaseUpdate::Set(
                        scrypto_encode(&blueprint_definition.auth_config.into_locked_substate())
                            .unwrap(),
                    ),
                );
                royalty_config_updates.insert(
                    SpreadPrefixKeyMapper::map_to_db_sort_key(
                        &scrypto_encode(&blueprint_version_key.clone()).unwrap(),
                    ),
                    DatabaseUpdate::Set(
                        scrypto_encode(&blueprint_definition.royalty_config.into_locked_substate())
                            .unwrap(),
                    ),
                );
            }

            let database_updates = indexmap!(
                blueprints_partition_key => blueprint_updates,
                dependencies_partition_key => dependency_updates,
                auth_configs_partition_key => auth_config_updates,
                royalty_configs_partition_key => royalty_config_updates,
                schemas_partition_key => schema_updates,
                vm_type_partition_key => vm_type_updates,
                original_code_partition_key => original_code_updates,
                instrumented_code_partition_key => instrumented_code_updates,
            );

            db.commit(&DatabaseUpdates::from_delta_maps(database_updates));

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
                manifest.into(),
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
