use crate::blueprints::models::KeyValueEntryContentSource;
use crate::internal_prelude::{
    KeyValueEntryPayload, PackageCodeOriginalCodeV1, PackageCodeVmTypeV1, PackageCollection,
    VersionedPackageBlueprintVersionDefinition, VersionedPackageCodeOriginalCode,
    VersionedPackageCodeVmTypeVersion,
};
use crate::system::system_db_reader::{ObjectCollectionKey, SystemDatabaseReader};
use crate::track::{NodeStateUpdates, PartitionStateUpdates, StateUpdates};
use radix_engine_common::constants::{BOOT_LOADER_STATE, CONSENSUS_MANAGER_PACKAGE};
use radix_engine_common::crypto::hash;
use radix_engine_common::prelude::ScopedTypeId;
use radix_engine_common::prelude::{scrypto_encode, ScryptoCustomTypeKind};
use radix_engine_common::types::SubstateKey;
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::consensus_manager::{
    ConsensusManagerCompareCurrentTimeInputV2, ConsensusManagerGetCurrentTimeInputV2,
    CONSENSUS_MANAGER_BLUEPRINT, CONSENSUS_MANAGER_COMPARE_CURRENT_TIME_IDENT,
    CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT,
};
use radix_engine_interface::blueprints::package::{
    BlueprintPayloadDef, BlueprintVersion, BlueprintVersionKey, CodeHash, VmType,
    CONSENSUS_MANAGER_SECONDS_PRECISION_CODE_ID,
};
use radix_engine_interface::prelude::HasSchemaHash;
use radix_engine_interface::prelude::IsHash;
use radix_engine_interface::prelude::ToString;
use radix_engine_interface::types::CollectionDescriptor;
use radix_engine_store_interface::interface::{DatabaseUpdate, SubstateDatabase};
use sbor::HasLatestVersion;
use sbor::{generate_full_schema, TypeAggregator};
use utils::indexmap;
use crate::vm::{BOOT_LOADER_VM_PARTITION_NUM, BOOT_LOADER_VM_SUBSTATE_FIELD_KEY, VmBoot};

pub fn generate_vm_boot_scrypto_minor_version_state_updates() -> StateUpdates {
    let substate = scrypto_encode(&VmBoot::V1 { scrypto_v1_minor_version: 1u64 }).unwrap();

    StateUpdates {
        by_node: indexmap!(
            BOOT_LOADER_STATE => NodeStateUpdates::Delta {
                by_partition: indexmap! {
                    BOOT_LOADER_VM_PARTITION_NUM => PartitionStateUpdates::Delta {
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
    let consensus_mgr_node_id = CONSENSUS_MANAGER_PACKAGE.into_node_id();
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
                &consensus_mgr_node_id,
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
            &consensus_mgr_node_id,
            ObjectModuleId::Main,
            PackageCollection::BlueprintVersionDefinitionKeyValue.collection_index(),
        )
        .unwrap();

    let code_vm_type_partition_num = reader
        .get_partition_of_collection(
            &consensus_mgr_node_id,
            ObjectModuleId::Main,
            PackageCollection::CodeVmTypeKeyValue.collection_index(),
        )
        .unwrap();

    let code_partition_num = reader
        .get_partition_of_collection(
            &consensus_mgr_node_id,
            ObjectModuleId::Main,
            PackageCollection::CodeOriginalCodeKeyValue.collection_index(),
        )
        .unwrap();

    let schema_partition_num = reader
        .get_partition_of_collection(
            &consensus_mgr_node_id,
            ObjectModuleId::Main,
            PackageCollection::SchemaKeyValue.collection_index(),
        )
        .unwrap();

    StateUpdates {
        by_node: indexmap!(
            consensus_mgr_node_id => NodeStateUpdates::Delta {
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
