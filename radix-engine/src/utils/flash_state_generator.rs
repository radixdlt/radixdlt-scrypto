use crate::blueprints::consensus_manager::*;
use crate::blueprints::models::KeyValueEntryContentSource;
use crate::blueprints::package::*;
use crate::blueprints::pool::v1::constants::*;
use crate::internal_prelude::*;
use crate::system::system_db_reader::{ObjectCollectionKey, SystemDatabaseReader};
use crate::track::{NodeStateUpdates, PartitionStateUpdates, StateUpdates};
use crate::vm::*;
use radix_engine_common::constants::*;
use radix_engine_common::crypto::hash;
use radix_engine_common::math::Decimal;
use radix_engine_common::prelude::ScopedTypeId;
use radix_engine_common::prelude::{scrypto_encode, ScryptoCustomTypeKind};
use radix_engine_common::types::SubstateKey;
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::prelude::*;
use radix_engine_interface::types::CollectionDescriptor;
use sbor::HasLatestVersion;
use sbor::{generate_full_schema, TypeAggregator};
use substate_store_interface::interface::*;
use utils::indexmap;

pub fn generate_vm_boot_scrypto_minor_version_state_updates() -> StateUpdates {
    let substate = scrypto_encode(&VmBoot::V1 {
        scrypto_v1_minor_version: 1u64,
    })
    .unwrap();

    StateUpdates {
        by_node: indexmap!(
            TRANSACTION_TRACKER.into_node_id() => NodeStateUpdates::Delta {
                by_partition: indexmap! {
                    BOOT_LOADER_PARTITION => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Field(BOOT_LOADER_VM_SUBSTATE_FIELD_KEY) => DatabaseUpdate::Set(substate)
                        }
                    },
                }
            }
        ),
    }
}

/// Generates the state updates required for updating the Consensus Manager blueprint
/// to use seconds precision
pub fn generate_seconds_precision_state_updates<S: SubstateDatabase>(db: &S) -> StateUpdates {
    let reader = SystemDatabaseReader::new(db);
    let consensus_mgr_pkg_node_id = CONSENSUS_MANAGER_PACKAGE.into_node_id();
    let bp_version_key = BlueprintVersionKey {
        blueprint: CONSENSUS_MANAGER_BLUEPRINT.to_string(),
        version: BlueprintVersion::default(),
    };

    // Generate the new code substates
    let (new_code_substate, new_vm_type_substate, code_hash) = {
        let original_code = CONSENSUS_MANAGER_SECONDS_PRECISION_CODE_ID
            .to_be_bytes()
            .to_vec();

        let code_hash = CodeHash::from_hash(hash(&original_code));
        let versioned_code = VersionedPackageCodeOriginalCode::V1(PackageCodeOriginalCodeV1 {
            code: original_code,
        });
        let code_payload = versioned_code.into_payload();
        let code_substate = code_payload.into_locked_substate();
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

        let mut definition = versioned_definition.into_latest();

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
            &VersionedPackageBlueprintVersionDefinition::V1(definition).into_locked_substate(),
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

/// Generates the state updates required to update the pool package from the v1.0 to the v1.1
/// logic. No schema changes took place, just a change of logic. It produces the following
/// updates:
///
/// * Removes the old code_hash => vm_type substate.
/// * Adds a new code_hash => vm_type substate.
/// * Removes the old code_hash => original_code substate.
/// * Adds a new code_hash => original_code substate.
/// * Updates the function exports in the blueprint definitions to point to the new code hash.
pub fn generate_pools_v1_1_state_updates<S: SubstateDatabase>(db: &S) -> StateUpdates {
    let reader = SystemDatabaseReader::new(db);

    let pool_package_node_id = POOL_PACKAGE.into_node_id();

    // The old and new code hashes
    let old_code_id = POOL_V1_0_CODE_ID;
    let new_code_id = POOL_V1_1_CODE_ID;

    let old_code = old_code_id.to_be_bytes().to_vec();
    let new_code = new_code_id.to_be_bytes().to_vec();

    let old_code_hash = CodeHash::from_hash(hash(&old_code));
    let new_code_hash = CodeHash::from_hash(hash(&new_code));

    // New code substate created from the new code
    let new_code_substate =
        VersionedPackageCodeOriginalCode::V1(PackageCodeOriginalCodeV1 { code: new_code })
            .into_payload()
            .into_locked_substate();

    // New VM substate, which we will map the new code hash to.
    let new_vm_type_substate = VersionedPackageCodeVmType::V1(PackageCodeVmTypeV1 {
        vm_type: VmType::Native,
    })
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
            let mut blueprint_definition = versioned_definition.into_latest();

            for (_, export) in blueprint_definition.function_exports.iter_mut() {
                export.code_hash = new_code_hash
            }

            (
                blueprint_version_key,
                VersionedPackageBlueprintVersionDefinition::V1(blueprint_definition)
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

pub fn generate_validator_fee_fix_state_updates<S: SubstateDatabase>(db: &S) -> StateUpdates {
    let reader = SystemDatabaseReader::new(db);
    let consensus_mgr_node_id = CONSENSUS_MANAGER.into_node_id();

    let versioned_config: VersionedConsensusManagerConfiguration = reader
        .read_typed_object_field(
            &consensus_mgr_node_id,
            ModuleId::Main,
            ConsensusManagerField::Configuration.field_index(),
        )
        .unwrap();

    let mut config = versioned_config.into_latest();
    config.config.validator_creation_usd_cost = Decimal::from(100);

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
