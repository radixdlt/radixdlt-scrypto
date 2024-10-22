use super::*;
use crate::blueprints::consensus_manager::*;
use crate::blueprints::models::KeyValueEntryContentSource;
use crate::blueprints::package::*;
use crate::blueprints::pool::v1::constants::*;
use crate::internal_prelude::*;
use crate::system::system_db_reader::*;
use crate::vm::*;
use sbor::{generate_full_schema, TypeAggregator};

#[derive(Clone, ScryptoSbor)]
pub struct AnemoneSettings {
    /// Changes the cost associated with validator creation.
    pub validator_fee_fix: UpdateSetting<AnemoneValidatorCreationFee>,

    /// Exposes second-precision timestamp.
    pub seconds_precision: UpdateSetting<NoSettings>,

    /// Introduces BLS12-381 and Keccak-256 features.
    pub vm_boot_to_enable_bls128_and_keccak256: UpdateSetting<NoSettings>,

    /// Increases the math precision with native pool implementations.
    pub pools_update: UpdateSetting<NoSettings>,
}

impl UpdateSettings for AnemoneSettings {
    type UpdateGenerator = AnemoneGenerator;

    fn protocol_version() -> ProtocolVersion {
        ProtocolVersion::Anemone
    }

    fn all_enabled_as_default_for_network(network: &NetworkDefinition) -> Self {
        Self {
            validator_fee_fix: UpdateSetting::enabled_as_default_for_network(network),
            seconds_precision: UpdateSetting::enabled_as_default_for_network(network),
            vm_boot_to_enable_bls128_and_keccak256: UpdateSetting::enabled_as_default_for_network(
                network,
            ),
            pools_update: UpdateSetting::enabled_as_default_for_network(network),
        }
    }

    fn all_disabled() -> Self {
        Self {
            validator_fee_fix: UpdateSetting::Disabled,
            seconds_precision: UpdateSetting::Disabled,
            vm_boot_to_enable_bls128_and_keccak256: UpdateSetting::Disabled,
            pools_update: UpdateSetting::Disabled,
        }
    }

    fn create_generator(&self) -> Self::UpdateGenerator {
        AnemoneGenerator {
            settings: self.clone(),
        }
    }
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct AnemoneValidatorCreationFee {
    pub usd_fee: Decimal,
}

impl UpdateSettingContent for AnemoneValidatorCreationFee {
    fn default_setting(network_definition: &NetworkDefinition) -> Self {
        let usd_fee = match network_definition.id {
            241 => dec!(1), // Node integration test network
            _ => dec!(100), // All others including mainnet
        };
        Self { usd_fee }
    }
}

pub struct AnemoneGenerator {
    settings: AnemoneSettings,
}

impl ProtocolUpdateGenerator for AnemoneGenerator {
    fn insert_status_tracking_flash_transactions(&self) -> bool {
        // This was launched without status tracking, so we can't add it in later to avoid divergence
        false
    }

    fn batch_groups(&self) -> Vec<Box<dyn ProtocolUpdateBatchGroupGenerator + '_>> {
        vec![FixedBatchGroupGenerator::named("principal")
            .add_batch("primary", |store| generate_batch(store, &self.settings))
            .build()]
    }
}

#[deny(unused_variables)]
fn generate_batch(
    store: &dyn SubstateDatabase,
    AnemoneSettings {
        validator_fee_fix,
        seconds_precision,
        vm_boot_to_enable_bls128_and_keccak256,
        pools_update,
    }: &AnemoneSettings,
) -> ProtocolUpdateBatch {
    let mut batch = ProtocolUpdateBatch::empty();

    if let UpdateSetting::Enabled(creation_fee) = &validator_fee_fix {
        batch.mut_add_flash(
            "anemone-validator-fee-fix",
            generate_validator_creation_fee_fix_state_updates(store, creation_fee),
        );
    }

    if let UpdateSetting::Enabled(NoSettings) = &seconds_precision {
        batch.mut_add_flash(
            "anemone-seconds-precision",
            generate_seconds_precision_timestamp_state_updates(store),
        );
    }

    if let UpdateSetting::Enabled(NoSettings) = &vm_boot_to_enable_bls128_and_keccak256 {
        batch.mut_add_flash(
            "anemone-vm-boot",
            generate_vm_boot_for_bls128_and_keccak256_state_updates(),
        );
    }

    if let UpdateSetting::Enabled(NoSettings) = &pools_update {
        batch.mut_add_flash(
            "anemone-pools",
            generate_pool_math_precision_fix_state_updates(store),
        );
    }

    batch
}

fn generate_validator_creation_fee_fix_state_updates<S: SubstateDatabase + ?Sized>(
    db: &S,
    validator_creation_fee: &AnemoneValidatorCreationFee,
) -> StateUpdates {
    let reader = SystemDatabaseReader::new(db);
    let consensus_mgr_node_id = CONSENSUS_MANAGER.into_node_id();

    let versioned_config: VersionedConsensusManagerConfiguration = reader
        .read_typed_object_field(
            &consensus_mgr_node_id,
            ModuleId::Main,
            ConsensusManagerField::Configuration.field_index(),
        )
        .unwrap();

    let mut config = versioned_config.fully_update_and_into_latest_version();
    config.config.validator_creation_usd_cost = validator_creation_fee.usd_fee;

    let updated_substate = config.into_locked_substate();

    StateUpdates {
        by_node: indexmap!(
            consensus_mgr_node_id => NodeStateUpdates::Delta {
                by_partition: indexmap! {
                    MAIN_BASE_PARTITION => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Field(ConsensusManagerField::Configuration.field_index()) => DatabaseUpdate::Set(
                                scrypto_encode(&updated_substate).unwrap()
                            )
                        }
                    },
                }
            }
        ),
    }
}

/// Generates the state updates required for updating the Consensus Manager blueprint
/// to use seconds precision
fn generate_seconds_precision_timestamp_state_updates<S: SubstateDatabase + ?Sized>(
    db: &S,
) -> StateUpdates {
    let reader = SystemDatabaseReader::new(db);
    let consensus_mgr_pkg_node_id = CONSENSUS_MANAGER_PACKAGE.into_node_id();
    let bp_version_key = BlueprintVersionKey {
        blueprint: CONSENSUS_MANAGER_BLUEPRINT.to_string(),
        version: BlueprintVersion::default(),
    };

    // Generate the new code substates
    let (new_code_substate, new_vm_type_substate, code_hash) = {
        let original_code = (NativeCodeId::ConsensusManagerCode2 as u64)
            .to_be_bytes()
            .to_vec();

        let code_hash = CodeHash::from_hash(hash(&original_code));
        let code_substate = PackageCodeOriginalCodeV1 {
            code: original_code,
        }
        .into_versioned()
        .into_locked_substate();
        let vm_type_substate = PackageCodeVmTypeV1 {
            vm_type: VmType::Native,
        }
        .into_versioned()
        .into_locked_substate();
        (
            scrypto_encode(&code_substate).unwrap(),
            scrypto_encode(&vm_type_substate).unwrap(),
            code_hash,
        )
    };

    // Generate the new schema substate
    let (
        new_schema_substate,
        get_current_time_input_v2_type_id,
        compare_current_time_input_v2_type_id,
        new_schema_hash,
    ) = {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let get_current_time_input_v2 =
            aggregator.add_child_type_and_descendents::<ConsensusManagerGetCurrentTimeInputV2>();
        let compare_current_time_input_v2 = aggregator
            .add_child_type_and_descendents::<ConsensusManagerCompareCurrentTimeInputV2>();
        let schema = generate_full_schema(aggregator);
        let schema_hash = schema.generate_schema_hash();
        let schema_substate = schema.into_locked_substate();
        (
            scrypto_encode(&schema_substate).unwrap(),
            get_current_time_input_v2,
            compare_current_time_input_v2,
            schema_hash,
        )
    };

    // Generate the blueprint definition substate updates
    let updated_bp_definition_substate = {
        let versioned_definition: VersionedPackageBlueprintVersionDefinition = reader
            .read_object_collection_entry(
                &consensus_mgr_pkg_node_id,
                ObjectModuleId::Main,
                ObjectCollectionKey::KeyValue(
                    PackageCollection::BlueprintVersionDefinitionKeyValue.collection_index(),
                    &bp_version_key,
                ),
            )
            .unwrap()
            .unwrap();

        let mut definition = versioned_definition.fully_update_and_into_latest_version();

        let export = definition
            .function_exports
            .get_mut(CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT)
            .unwrap();
        export.code_hash = code_hash;
        let function_schema = definition
            .interface
            .functions
            .get_mut(CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT)
            .unwrap();
        function_schema.input = BlueprintPayloadDef::Static(ScopedTypeId(
            new_schema_hash,
            get_current_time_input_v2_type_id,
        ));

        let export = definition
            .function_exports
            .get_mut(CONSENSUS_MANAGER_COMPARE_CURRENT_TIME_IDENT)
            .unwrap();
        export.code_hash = code_hash;
        let function_schema = definition
            .interface
            .functions
            .get_mut(CONSENSUS_MANAGER_COMPARE_CURRENT_TIME_IDENT)
            .unwrap();
        function_schema.input = BlueprintPayloadDef::Static(ScopedTypeId(
            new_schema_hash,
            compare_current_time_input_v2_type_id,
        ));

        scrypto_encode(
            &PackageBlueprintVersionDefinitionVersions::V1(definition)
                .into_versioned()
                .into_locked_substate(),
        )
        .unwrap()
    };

    let bp_definition_partition_num = reader
        .get_partition_of_collection(
            &consensus_mgr_pkg_node_id,
            ObjectModuleId::Main,
            PackageCollection::BlueprintVersionDefinitionKeyValue.collection_index(),
        )
        .unwrap();

    let code_vm_type_partition_num = reader
        .get_partition_of_collection(
            &consensus_mgr_pkg_node_id,
            ObjectModuleId::Main,
            PackageCollection::CodeVmTypeKeyValue.collection_index(),
        )
        .unwrap();

    let code_partition_num = reader
        .get_partition_of_collection(
            &consensus_mgr_pkg_node_id,
            ObjectModuleId::Main,
            PackageCollection::CodeOriginalCodeKeyValue.collection_index(),
        )
        .unwrap();

    let schema_partition_num = reader
        .get_partition_of_collection(
            &consensus_mgr_pkg_node_id,
            ObjectModuleId::Main,
            PackageCollection::SchemaKeyValue.collection_index(),
        )
        .unwrap();

    StateUpdates {
        by_node: indexmap!(
            consensus_mgr_pkg_node_id => NodeStateUpdates::Delta {
                by_partition: indexmap! {
                    bp_definition_partition_num => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Map(scrypto_encode(&bp_version_key).unwrap()) => DatabaseUpdate::Set(
                                updated_bp_definition_substate
                            )
                        }
                    },
                    code_vm_type_partition_num => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Map(scrypto_encode(&code_hash).unwrap()) => DatabaseUpdate::Set(new_vm_type_substate)
                        }
                    },
                    code_partition_num => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Map(scrypto_encode(&code_hash).unwrap()) => DatabaseUpdate::Set(new_code_substate)
                        }
                    },
                    schema_partition_num => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Map(scrypto_encode(&new_schema_hash).unwrap()) => DatabaseUpdate::Set(new_schema_substate)
                        }
                    }
                }
            }
        ),
    }
}

fn generate_vm_boot_for_bls128_and_keccak256_state_updates() -> StateUpdates {
    StateUpdates::empty().set_substate(
        TRANSACTION_TRACKER,
        BOOT_LOADER_PARTITION,
        BootLoaderField::VmBoot,
        VmBoot::V1 {
            scrypto_version: ScryptoVmVersion::crypto_utils_v1().into(),
        },
    )
}

/// Generates the state updates required to update the pool package from the v1.0 to the v1.1
/// logic. No schema changes took place, just a change of logic. It produces the following
/// updates:
///
/// * Removes the old code_hash => vm_type substate.
/// * Adds a new code_hash => vm_type substate.
/// * Removes the old code_hash => original_code substate.
/// * Adds a new code_hash => original_code substate.
/// * Updates the function exports in the blueprint definitions to point to the new code hash.
fn generate_pool_math_precision_fix_state_updates<S: SubstateDatabase + ?Sized>(
    db: &S,
) -> StateUpdates {
    let reader = SystemDatabaseReader::new(db);

    let pool_package_node_id = POOL_PACKAGE.into_node_id();

    // The old and new code hashes
    let old_code_id = NativeCodeId::PoolCode1;
    let new_code_id = NativeCodeId::PoolCode2;

    let old_code = (old_code_id as u64).to_be_bytes().to_vec();
    let new_code = (new_code_id as u64).to_be_bytes().to_vec();

    let old_code_hash = CodeHash::from_hash(hash(&old_code));
    let new_code_hash = CodeHash::from_hash(hash(&new_code));

    // New code substate created from the new code
    let new_code_substate = PackageCodeOriginalCodeV1 { code: new_code }
        .into_versioned()
        .into_payload()
        .into_locked_substate();

    // New VM substate, which we will map the new code hash to.
    let new_vm_type_substate = PackageCodeVmTypeV1 {
        vm_type: VmType::Native,
    }
    .into_versioned()
    .into_payload()
    .into_locked_substate();

    // Update the function exports in the blueprint definition.
    let [(one_resource_pool_blueprint_key, one_resource_pool_blueprint_definition), (two_resource_pool_blueprint_key, two_resource_pool_blueprint_definition), (multi_resource_pool_blueprint_key, multi_resource_pool_blueprint_definition)] =
        [
            ONE_RESOURCE_POOL_BLUEPRINT_IDENT,
            TWO_RESOURCE_POOL_BLUEPRINT_IDENT,
            MULTI_RESOURCE_POOL_BLUEPRINT_IDENT,
        ]
        .map(|blueprint_name| {
            let blueprint_version_key = BlueprintVersionKey {
                blueprint: blueprint_name.to_owned(),
                version: BlueprintVersion::default(),
            };

            let versioned_definition: VersionedPackageBlueprintVersionDefinition = reader
                .read_object_collection_entry(
                    &pool_package_node_id,
                    ObjectModuleId::Main,
                    ObjectCollectionKey::KeyValue(
                        PackageCollection::BlueprintVersionDefinitionKeyValue.collection_index(),
                        &blueprint_version_key,
                    ),
                )
                .unwrap()
                .unwrap();
            let mut blueprint_definition =
                versioned_definition.fully_update_and_into_latest_version();

            for (_, export) in blueprint_definition.function_exports.iter_mut() {
                export.code_hash = new_code_hash
            }

            (
                blueprint_version_key,
                PackageBlueprintVersionDefinitionVersions::V1(blueprint_definition)
                    .into_versioned()
                    .into_payload()
                    .into_locked_substate(),
            )
        });

    let original_code_partition_number = reader
        .get_partition_of_collection(
            &pool_package_node_id,
            ObjectModuleId::Main,
            PackageCollection::CodeOriginalCodeKeyValue.collection_index(),
        )
        .unwrap();

    let code_vm_type_partition_number = reader
        .get_partition_of_collection(
            &pool_package_node_id,
            ObjectModuleId::Main,
            PackageCollection::CodeVmTypeKeyValue.collection_index(),
        )
        .unwrap();

    let blueprint_definition_partition_number = reader
        .get_partition_of_collection(
            &pool_package_node_id,
            ObjectModuleId::Main,
            PackageCollection::BlueprintVersionDefinitionKeyValue.collection_index(),
        )
        .unwrap();

    StateUpdates {
        by_node: indexmap! {
            pool_package_node_id => NodeStateUpdates::Delta {
                by_partition: indexmap! {
                    original_code_partition_number => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Map(scrypto_encode(&old_code_hash).unwrap())
                                => DatabaseUpdate::Delete,
                            SubstateKey::Map(scrypto_encode(&new_code_hash).unwrap())
                                => DatabaseUpdate::Set(scrypto_encode(&new_code_substate).unwrap()),
                        }
                    },
                    code_vm_type_partition_number => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Map(scrypto_encode(&old_code_hash).unwrap())
                                => DatabaseUpdate::Delete,
                            SubstateKey::Map(scrypto_encode(&new_code_hash).unwrap())
                                => DatabaseUpdate::Set(scrypto_encode(&new_vm_type_substate).unwrap()),
                        }
                    },
                    blueprint_definition_partition_number => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Map(scrypto_encode(&one_resource_pool_blueprint_key).unwrap())
                                => DatabaseUpdate::Set(scrypto_encode(&one_resource_pool_blueprint_definition).unwrap()),
                            SubstateKey::Map(scrypto_encode(&two_resource_pool_blueprint_key).unwrap())
                                => DatabaseUpdate::Set(scrypto_encode(&two_resource_pool_blueprint_definition).unwrap()),
                            SubstateKey::Map(scrypto_encode(&multi_resource_pool_blueprint_key).unwrap())
                                => DatabaseUpdate::Set(scrypto_encode(&multi_resource_pool_blueprint_definition).unwrap()),
                        }
                    }
                }
            }
        },
    }
}
