use super::*;
use crate::blueprints::access_controller::v1::AccessControllerV1NativePackage;
use crate::blueprints::access_controller::v2::AccessControllerV2NativePackage;
use crate::blueprints::locker::LockerNativePackage;
use crate::blueprints::models::KeyValueEntryContentSource;
use crate::blueprints::package::*;
use crate::internal_prelude::*;
use crate::kernel::kernel::*;
use crate::object_modules::role_assignment::*;
use crate::system::system_callback::*;
use crate::system::system_db_reader::*;
use crate::vm::*;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::transaction_processor::*;

#[derive(Clone, ScryptoSbor)]
pub struct BottlenoseSettings {
    /// Exposes a getter method for reading owner role rule.
    pub add_owner_role_getter: UpdateSetting<NoSettings>,

    /// Introduces the account locker blueprint.
    pub add_locker_package: UpdateSetting<NoSettings>,

    /// Makes some behavioral changes to the try_deposit_or_refund (and batch variants too) method
    /// on the account blueprint.
    pub fix_account_try_deposit_or_refund_behaviour: UpdateSetting<NoSettings>,

    /// Moves various protocol parameters to state.
    pub move_protocol_params_to_state: UpdateSetting<ProtocolParamsSettings>,

    /// Adds an XRD vault to the access controller for locking fees.
    pub update_access_controller_to_add_xrd_fee_vault: UpdateSetting<NoSettings>,

    /// Imposes a limits on the blobs in the transaction processor
    pub impose_a_limit_on_transaction_processor_blobs: UpdateSetting<NoSettings>,

    /// Adds differed reference cost checks.    
    pub apply_costing_for_ref_checks: UpdateSetting<NoSettings>,

    /// Add restrictions to use of role key in role list.
    pub restrict_reserved_role_key: UpdateSetting<NoSettings>,
}

#[derive(Clone, Sbor)]
pub struct ProtocolParamsSettings {
    pub network_definition: NetworkDefinition,
}

impl UpdateSettingContent for ProtocolParamsSettings {
    fn default_setting(network_definition: &NetworkDefinition) -> Self {
        Self {
            network_definition: network_definition.clone(),
        }
    }
}

impl UpdateSettings for BottlenoseSettings {
    type UpdateGenerator = BottlenoseGenerator;

    fn protocol_version() -> ProtocolVersion {
        ProtocolVersion::Bottlenose
    }

    fn all_enabled_as_default_for_network(network: &NetworkDefinition) -> Self {
        Self {
            add_owner_role_getter: UpdateSetting::enabled_as_default_for_network(network),
            add_locker_package: UpdateSetting::enabled_as_default_for_network(network),
            move_protocol_params_to_state: UpdateSetting::enabled_as_default_for_network(network),
            fix_account_try_deposit_or_refund_behaviour:
                UpdateSetting::enabled_as_default_for_network(network),
            update_access_controller_to_add_xrd_fee_vault:
                UpdateSetting::enabled_as_default_for_network(network),
            impose_a_limit_on_transaction_processor_blobs:
                UpdateSetting::enabled_as_default_for_network(network),
            apply_costing_for_ref_checks: UpdateSetting::enabled_as_default_for_network(network),
            restrict_reserved_role_key: UpdateSetting::enabled_as_default_for_network(network),
        }
    }

    fn all_disabled() -> Self {
        Self {
            add_owner_role_getter: UpdateSetting::Disabled,
            add_locker_package: UpdateSetting::Disabled,
            move_protocol_params_to_state: UpdateSetting::Disabled,
            fix_account_try_deposit_or_refund_behaviour: UpdateSetting::Disabled,
            update_access_controller_to_add_xrd_fee_vault: UpdateSetting::Disabled,
            impose_a_limit_on_transaction_processor_blobs: UpdateSetting::Disabled,
            apply_costing_for_ref_checks: UpdateSetting::Disabled,
            restrict_reserved_role_key: UpdateSetting::Disabled,
        }
    }

    fn create_generator(&self) -> Self::UpdateGenerator {
        BottlenoseGenerator {
            settings: self.clone(),
        }
    }
}

pub struct BottlenoseGenerator {
    settings: BottlenoseSettings,
}

impl ProtocolUpdateGenerator for BottlenoseGenerator {
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
    BottlenoseSettings {
        add_owner_role_getter,
        add_locker_package,
        fix_account_try_deposit_or_refund_behaviour,
        move_protocol_params_to_state,
        update_access_controller_to_add_xrd_fee_vault,
        impose_a_limit_on_transaction_processor_blobs,
        apply_costing_for_ref_checks: ref_cost_checks,
        restrict_reserved_role_key,
    }: &BottlenoseSettings,
) -> ProtocolUpdateBatch {
    let mut transactions = vec![];
    if let UpdateSetting::Enabled(NoSettings) = &add_owner_role_getter {
        transactions.push(ProtocolUpdateTransaction::flash(
            "bottlenose-owner-role-getter",
            generate_owner_role_getter_state_updates(store),
        ));
    }
    if let UpdateSetting::Enabled(NoSettings) = &add_locker_package {
        transactions.push(ProtocolUpdateTransaction::flash(
            "bottlenose-locker-package",
            generate_locker_package_state_updates(),
        ));
    }
    if let UpdateSetting::Enabled(NoSettings) = &fix_account_try_deposit_or_refund_behaviour {
        transactions.push(ProtocolUpdateTransaction::flash(
            "bottlenose-account-try-deposit-or-refund",
            generate_account_bottlenose_extension_state_updates(store),
        ));
    }
    if let UpdateSetting::Enabled(settings) = &move_protocol_params_to_state {
        transactions.push(ProtocolUpdateTransaction::flash(
            "bottlenose-protocol-params-to-state",
            generate_protocol_params_to_state_updates(settings.network_definition.clone()),
        ));
    }
    if let UpdateSetting::Enabled(NoSettings) = &update_access_controller_to_add_xrd_fee_vault {
        transactions.push(ProtocolUpdateTransaction::flash(
            "bottlenose-access-controller-xrd-fee-vault",
            generate_access_controller_state_updates(store),
        ));
    }
    if let UpdateSetting::Enabled(NoSettings) = &impose_a_limit_on_transaction_processor_blobs {
        transactions.push(ProtocolUpdateTransaction::flash(
            "bottlenose-transaction-processor-blob-limits",
            generate_transaction_processor_blob_limits_state_updates(store),
        ));
    }
    if let UpdateSetting::Enabled(NoSettings) = &ref_cost_checks {
        transactions.push(ProtocolUpdateTransaction::flash(
            "bottlenose-add-deferred-reference-check-cost",
            generate_ref_check_costs_state_updates(),
        ));
    }
    if let UpdateSetting::Enabled(NoSettings) = &restrict_reserved_role_key {
        transactions.push(ProtocolUpdateTransaction::flash(
            "bottlenose-restrict-role-assignment-reserved-role-key",
            generate_restrict_reserved_role_key_state_updates(store),
        ));
    }
    ProtocolUpdateBatch { transactions }
}

/// A quick macro for encoding and unwrapping.
macro_rules! scrypto_encode {
    (
        $expr: expr
    ) => {
        ::radix_common::prelude::scrypto_encode($expr).unwrap()
    };
}

fn generate_owner_role_getter_state_updates<S: SubstateDatabase + ?Sized>(db: &S) -> StateUpdates {
    let reader = SystemDatabaseReader::new(db);
    let node_id = ROLE_ASSIGNMENT_MODULE_PACKAGE.into_node_id();
    let blueprint_version_key = BlueprintVersionKey {
        blueprint: ROLE_ASSIGNMENT_BLUEPRINT.to_string(),
        version: Default::default(),
    };

    // Creating the original code substates for extension.
    let (code_hash, (code_substate, vm_type_substate)) = {
        let original_code = (NativeCodeId::RoleAssignmentCode2 as u64)
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
        .into_locked_substate();

        (code_hash, (code_substate, vm_type_substate))
    };

    // Creating the new schema substate with the methods added by the extension
    let (added_functions, schema) = RoleAssignmentBottlenoseExtension::added_functions_schema();
    let (schema_hash, schema_substate) =
        (schema.generate_schema_hash(), schema.into_locked_substate());

    // Updating the blueprint definition of the existing blueprint with the added functions.
    let blueprint_definition_substate = {
        let mut blueprint_definition = reader
            .read_object_collection_entry::<_, VersionedPackageBlueprintVersionDefinition>(
                &node_id,
                ObjectModuleId::Main,
                ObjectCollectionKey::KeyValue(
                    PackageCollection::BlueprintVersionDefinitionKeyValue.collection_index(),
                    &blueprint_version_key,
                ),
            )
            .unwrap()
            .unwrap()
            .fully_update_and_into_latest_version();

        for (function_name, added_function) in added_functions.into_iter() {
            let TypeRef::Static(input_local_id) = added_function.input else {
                unreachable!()
            };
            let TypeRef::Static(output_local_id) = added_function.output else {
                unreachable!()
            };

            blueprint_definition.function_exports.insert(
                function_name.clone(),
                PackageExport {
                    code_hash,
                    export_name: function_name.clone(),
                },
            );
            blueprint_definition.interface.functions.insert(
                function_name,
                FunctionSchema {
                    receiver: added_function.receiver,
                    input: BlueprintPayloadDef::Static(ScopedTypeId(schema_hash, input_local_id)),
                    output: BlueprintPayloadDef::Static(ScopedTypeId(schema_hash, output_local_id)),
                },
            );
        }

        blueprint_definition.into_locked_substate()
    };

    // Getting the partition number of the various collections that we're updating
    let [blueprint_version_definition_partition_number, code_vm_type_partition_number, code_original_code_partition_number, schema_partition_number] =
        [
            PackageCollection::BlueprintVersionDefinitionKeyValue,
            PackageCollection::CodeVmTypeKeyValue,
            PackageCollection::CodeOriginalCodeKeyValue,
            PackageCollection::SchemaKeyValue,
        ]
        .map(|package_collection| {
            reader
                .get_partition_of_collection(
                    &node_id,
                    ObjectModuleId::Main,
                    package_collection.collection_index(),
                )
                .unwrap()
        });

    // Generating the state updates
    StateUpdates {
        by_node: indexmap!(
            node_id => NodeStateUpdates::Delta {
                by_partition: indexmap! {
                    blueprint_version_definition_partition_number => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Map(scrypto_encode!(&blueprint_version_key)) => DatabaseUpdate::Set(
                                scrypto_encode!(&blueprint_definition_substate)
                            )
                        }
                    },
                    code_vm_type_partition_number => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Map(scrypto_encode!(&code_hash)) => DatabaseUpdate::Set(
                                scrypto_encode!(&vm_type_substate)
                            )
                        }
                    },
                    code_original_code_partition_number => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Map(scrypto_encode!(&code_hash)) => DatabaseUpdate::Set(
                                scrypto_encode!(&code_substate)
                            )
                        }
                    },
                    schema_partition_number => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Map(scrypto_encode!(&schema_hash)) => DatabaseUpdate::Set(
                                scrypto_encode!(&schema_substate)
                            )
                        }
                    }
                }
            }
        ),
    }
}

fn generate_locker_package_state_updates() -> StateUpdates {
    let package_definition = LockerNativePackage::definition();
    let package_structure = PackageNativePackage::validate_and_build_package_structure(
        package_definition,
        VmType::Native,
        (NativeCodeId::LockerCode1 as u64).to_be_bytes().to_vec(),
        Default::default(),
        false,
        &VmBoot::latest(),
    )
    .unwrap_or_else(|err| {
        panic!(
            "Invalid flashed Package definition with native_code_id {}: {:?}",
            NativeCodeId::LockerCode1 as u64,
            err
        )
    });

    let partitions = create_package_partition_substates(
        package_structure,
        metadata_init! {
            "name" => "Locker Package", locked;
            "description" => "A native package that defines the logic for dApp-owned lockers to send resources to specified account addresses.", locked;
        },
        None,
    );

    StateUpdates {
        by_node: indexmap! {
            LOCKER_PACKAGE.into_node_id() => NodeStateUpdates::Delta {
                by_partition: partitions
                    .into_iter()
                    .map(|(partition_num, substates)| {
                        (
                            partition_num,
                            PartitionStateUpdates::Delta {
                                by_substate: substates
                                    .into_iter()
                                    .map(|(key, value)| {
                                        (key, DatabaseUpdate::Set(value.as_vec_ref().clone()))
                                    })
                                    .collect(),
                            },
                        )
                    })
                    .collect(),
            }
        },
    }
}

fn generate_account_bottlenose_extension_state_updates<S: SubstateDatabase + ?Sized>(
    db: &S,
) -> StateUpdates {
    let reader = SystemDatabaseReader::new(db);
    let node_id = ACCOUNT_PACKAGE.into_node_id();
    let blueprint_version_key = BlueprintVersionKey {
        blueprint: ACCOUNT_BLUEPRINT.to_string(),
        version: Default::default(),
    };

    // Creating the original code substates for extension.
    let (code_hash, (code_substate, vm_type_substate)) = {
        let original_code = (NativeCodeId::AccountCode2 as u64).to_be_bytes().to_vec();

        let code_hash = CodeHash::from_hash(hash(&original_code));
        let code_substate = PackageCodeOriginalCodeV1 {
            code: original_code,
        }
        .into_locked_substate();
        let vm_type_substate = PackageCodeVmTypeV1 {
            vm_type: VmType::Native,
        }
        .into_locked_substate();

        (code_hash, (code_substate, vm_type_substate))
    };

    // Updating the blueprint definition of the existing blueprint so that the code used is the new
    // one.
    let blueprint_definition_substate = {
        let mut blueprint_definition = reader
            .read_object_collection_entry::<_, VersionedPackageBlueprintVersionDefinition>(
                &node_id,
                ObjectModuleId::Main,
                ObjectCollectionKey::KeyValue(
                    PackageCollection::BlueprintVersionDefinitionKeyValue.collection_index(),
                    &blueprint_version_key,
                ),
            )
            .unwrap()
            .unwrap()
            .fully_update_and_into_latest_version();

        for function_name in [
            ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT,
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT,
        ] {
            blueprint_definition
                .function_exports
                .get_mut(function_name)
                .expect("This function must exist")
                .code_hash = code_hash;
        }

        blueprint_definition.into_locked_substate()
    };

    // Getting the partition number of the various collections that we're updating
    let [blueprint_version_definition_partition_number, code_vm_type_partition_number, code_original_code_partition_number] =
        [
            PackageCollection::BlueprintVersionDefinitionKeyValue,
            PackageCollection::CodeVmTypeKeyValue,
            PackageCollection::CodeOriginalCodeKeyValue,
        ]
        .map(|package_collection| {
            reader
                .get_partition_of_collection(
                    &node_id,
                    ObjectModuleId::Main,
                    package_collection.collection_index(),
                )
                .unwrap()
        });

    // Generating the state updates
    StateUpdates {
        by_node: indexmap!(
            node_id => NodeStateUpdates::Delta {
                by_partition: indexmap! {
                    blueprint_version_definition_partition_number => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Map(scrypto_encode!(&blueprint_version_key)) => DatabaseUpdate::Set(
                                scrypto_encode!(&blueprint_definition_substate)
                            )
                        }
                    },
                    code_vm_type_partition_number => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Map(scrypto_encode!(&code_hash)) => DatabaseUpdate::Set(
                                scrypto_encode!(&vm_type_substate)
                            )
                        }
                    },
                    code_original_code_partition_number => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Map(scrypto_encode!(&code_hash)) => DatabaseUpdate::Set(
                                scrypto_encode!(&code_substate)
                            )
                        }
                    },
                }
            }
        ),
    }
}

fn generate_protocol_params_to_state_updates(
    network_definition: NetworkDefinition,
) -> StateUpdates {
    StateUpdates::empty().set_substate(
        TRANSACTION_TRACKER,
        BOOT_LOADER_PARTITION,
        BootLoaderField::SystemBoot,
        SystemBoot::bottlenose(network_definition),
    )
}

fn generate_access_controller_state_updates<S: SubstateDatabase + ?Sized>(db: &S) -> StateUpdates {
    let reader = SystemDatabaseReader::new(db);
    let node_id = ACCESS_CONTROLLER_PACKAGE.into_node_id();
    let blueprint_version_key = BlueprintVersionKey {
        blueprint: ACCESS_CONTROLLER_BLUEPRINT.to_string(),
        version: Default::default(),
    };
    let old_blueprint_definition = AccessControllerV1NativePackage::definition()
        .blueprints
        .swap_remove(ACCESS_CONTROLLER_BLUEPRINT)
        .unwrap();
    let new_blueprint_definition = AccessControllerV2NativePackage::definition()
        .blueprints
        .swap_remove(ACCESS_CONTROLLER_BLUEPRINT)
        .unwrap();

    let old_schema_hash = old_blueprint_definition
        .schema
        .schema
        .generate_schema_hash();
    let new_schema_hash = new_blueprint_definition
        .schema
        .schema
        .generate_schema_hash();
    let new_schema_substate = new_blueprint_definition
        .schema
        .schema
        .clone()
        .into_locked_substate();

    // Creating the original code substates for extension.
    let old_code_hash = CodeHash::from_hash(hash(
        (NativeCodeId::AccessControllerCode1 as u64)
            .to_be_bytes()
            .to_vec(),
    ));
    let (new_code_hash, (new_code_substate, new_vm_type_substate)) = {
        let original_code = (NativeCodeId::AccessControllerCode2 as u64)
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
        .into_locked_substate();

        (code_hash, (code_substate, vm_type_substate))
    };

    let new_blueprint_auth_config = new_blueprint_definition.auth_config.into_locked_substate();

    let blueprint_definition_substate = {
        let mut blueprint_definition = reader
            .read_object_collection_entry::<_, VersionedPackageBlueprintVersionDefinition>(
                &node_id,
                ObjectModuleId::Main,
                ObjectCollectionKey::KeyValue(
                    PackageCollection::BlueprintVersionDefinitionKeyValue.collection_index(),
                    &blueprint_version_key,
                ),
            )
            .unwrap()
            .unwrap()
            .fully_update_and_into_latest_version();

        blueprint_definition.interface.state = IndexedStateSchema::from_schema(
            new_blueprint_definition
                .schema
                .schema
                .generate_schema_hash(),
            new_blueprint_definition.schema.state,
            Default::default(),
        );

        blueprint_definition.interface.functions = new_blueprint_definition
            .schema
            .functions
            .clone()
            .functions
            .into_iter()
            .map(|(ident, func)| {
                (
                    ident,
                    FunctionSchema {
                        receiver: func.receiver,
                        input: BlueprintPayloadDef::Static(ScopedTypeId(
                            new_schema_hash,
                            func.input.assert_static(),
                        )),
                        output: BlueprintPayloadDef::Static(ScopedTypeId(
                            new_schema_hash,
                            func.output.assert_static(),
                        )),
                    },
                )
            })
            .collect();

        blueprint_definition.function_exports = new_blueprint_definition
            .schema
            .functions
            .clone()
            .functions
            .into_iter()
            .map(|(ident, func)| {
                (
                    ident,
                    PackageExport {
                        code_hash: new_code_hash,
                        export_name: func.export,
                    },
                )
            })
            .collect();

        blueprint_definition.interface.events = new_blueprint_definition
            .schema
            .events
            .clone()
            .event_schema
            .into_iter()
            .map(|(ident, type_ref)| {
                (
                    ident,
                    BlueprintPayloadDef::Static(ScopedTypeId(
                        new_schema_hash,
                        type_ref.assert_static(),
                    )),
                )
            })
            .collect();

        blueprint_definition.into_locked_substate()
    };

    let original_code_partition_number = reader
        .get_partition_of_collection(
            &node_id,
            ObjectModuleId::Main,
            PackageCollection::CodeOriginalCodeKeyValue.collection_index(),
        )
        .unwrap();

    let code_vm_type_partition_number = reader
        .get_partition_of_collection(
            &node_id,
            ObjectModuleId::Main,
            PackageCollection::CodeVmTypeKeyValue.collection_index(),
        )
        .unwrap();

    let blueprint_definition_partition_number = reader
        .get_partition_of_collection(
            &node_id,
            ObjectModuleId::Main,
            PackageCollection::BlueprintVersionDefinitionKeyValue.collection_index(),
        )
        .unwrap();

    let blueprint_auth_configs = reader
        .get_partition_of_collection(
            &node_id,
            ObjectModuleId::Main,
            PackageCollection::BlueprintVersionAuthConfigKeyValue.collection_index(),
        )
        .unwrap();

    let schema_partition_num = reader
        .get_partition_of_collection(
            &node_id,
            ObjectModuleId::Main,
            PackageCollection::SchemaKeyValue.collection_index(),
        )
        .unwrap();

    StateUpdates {
        by_node: indexmap! {
            node_id => NodeStateUpdates::Delta {
                by_partition: indexmap! {
                    blueprint_definition_partition_number => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Map(scrypto_encode!(&blueprint_version_key)) => DatabaseUpdate::Set(
                                scrypto_encode!(&blueprint_definition_substate)
                            )
                        }
                    },
                    blueprint_auth_configs => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Map(scrypto_encode!(&blueprint_version_key)) => DatabaseUpdate::Set(
                                scrypto_encode!(&new_blueprint_auth_config)
                            )
                        }
                    },
                    original_code_partition_number => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Map(scrypto_encode!(&old_code_hash))
                                => DatabaseUpdate::Delete,
                            SubstateKey::Map(scrypto_encode!(&new_code_hash))
                                => DatabaseUpdate::Set(scrypto_encode!(&new_code_substate)),
                        }
                    },
                    code_vm_type_partition_number => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Map(scrypto_encode!(&old_code_hash))
                                => DatabaseUpdate::Delete,
                            SubstateKey::Map(scrypto_encode!(&new_code_hash))
                                => DatabaseUpdate::Set(scrypto_encode!(&new_vm_type_substate)),
                        }
                    },
                    schema_partition_num => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Map(scrypto_encode!(&old_schema_hash))
                                => DatabaseUpdate::Delete,
                            SubstateKey::Map(scrypto_encode!(&new_schema_hash))
                                => DatabaseUpdate::Set(scrypto_encode!(&new_schema_substate))
                        }
                    }
                }
            }
        },
    }
}

/// Generates the state updates required for restricting reserved role key.
fn generate_restrict_reserved_role_key_state_updates<S: SubstateDatabase + ?Sized>(
    db: &S,
) -> StateUpdates {
    let reader = SystemDatabaseReader::new(db);
    let tx_processor_pkg_node_id = PACKAGE_PACKAGE.into_node_id();
    let bp_version_key = BlueprintVersionKey {
        blueprint: PACKAGE_BLUEPRINT.to_string(),
        version: BlueprintVersion::default(),
    };

    // Generate the new code substates
    let (new_code_substate, new_vm_type_substate, old_code_hash, new_code_hash) = {
        let old_code = (NativeCodeId::PackageCode1 as u64).to_be_bytes().to_vec();
        let old_code_hash = CodeHash::from_hash(hash(&old_code));

        let new_code = (NativeCodeId::PackageCode2 as u64).to_be_bytes().to_vec();
        let new_code_hash = CodeHash::from_hash(hash(&new_code));

        let versioned_code = PackageCodeOriginalCodeV1 { code: new_code }.into_versioned();
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
            old_code_hash,
            new_code_hash,
        )
    };

    // Generate the blueprint definition substate updates
    let updated_bp_definition_substate = {
        let versioned_definition: VersionedPackageBlueprintVersionDefinition = reader
            .read_object_collection_entry(
                &tx_processor_pkg_node_id,
                ObjectModuleId::Main,
                ObjectCollectionKey::KeyValue(
                    PackageCollection::BlueprintVersionDefinitionKeyValue.collection_index(),
                    &bp_version_key,
                ),
            )
            .unwrap()
            .unwrap();

        let mut definition = versioned_definition.fully_update_and_into_latest_version();

        for (_, export) in definition.function_exports.iter_mut() {
            export.code_hash = new_code_hash
        }

        scrypto_encode(&definition.into_versioned().into_locked_substate()).unwrap()
    };

    let bp_definition_partition_num = reader
        .get_partition_of_collection(
            &tx_processor_pkg_node_id,
            ObjectModuleId::Main,
            PackageCollection::BlueprintVersionDefinitionKeyValue.collection_index(),
        )
        .unwrap();

    let vm_type_partition_num = reader
        .get_partition_of_collection(
            &tx_processor_pkg_node_id,
            ObjectModuleId::Main,
            PackageCollection::CodeVmTypeKeyValue.collection_index(),
        )
        .unwrap();

    let original_code_partition_num = reader
        .get_partition_of_collection(
            &tx_processor_pkg_node_id,
            ObjectModuleId::Main,
            PackageCollection::CodeOriginalCodeKeyValue.collection_index(),
        )
        .unwrap();

    StateUpdates {
        by_node: indexmap!(
            tx_processor_pkg_node_id => NodeStateUpdates::Delta {
                by_partition: indexmap! {
                    bp_definition_partition_num => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Map(scrypto_encode(&bp_version_key).unwrap()) => DatabaseUpdate::Set(
                                updated_bp_definition_substate
                            )
                        }
                    },
                    vm_type_partition_num => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Map(scrypto_encode(&old_code_hash).unwrap()) => DatabaseUpdate::Delete,
                            SubstateKey::Map(scrypto_encode(&new_code_hash).unwrap()) => DatabaseUpdate::Set(new_vm_type_substate)
                        }
                    },
                    original_code_partition_num => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Map(scrypto_encode(&old_code_hash).unwrap()) => DatabaseUpdate::Delete,
                            SubstateKey::Map(scrypto_encode(&new_code_hash).unwrap()) => DatabaseUpdate::Set(new_code_substate)
                        }
                    },
                }
            }
        ),
    }
}

/// Generates the state updates required for updating the TransactionProcessor blueprint
/// to limit blob memory usage
fn generate_transaction_processor_blob_limits_state_updates<S: SubstateDatabase + ?Sized>(
    db: &S,
) -> StateUpdates {
    let reader = SystemDatabaseReader::new(db);
    let tx_processor_pkg_node_id = TRANSACTION_PROCESSOR_PACKAGE.into_node_id();
    let bp_version_key = BlueprintVersionKey {
        blueprint: TRANSACTION_PROCESSOR_BLUEPRINT.to_string(),
        version: BlueprintVersion::default(),
    };

    // Generate the new code substates
    let (new_code_substate, new_vm_type_substate, old_code_hash, new_code_hash) = {
        let old_code = (NativeCodeId::TransactionProcessorCode1 as u64)
            .to_be_bytes()
            .to_vec();
        let old_code_hash = CodeHash::from_hash(hash(&old_code));

        let new_code = (NativeCodeId::TransactionProcessorCode2 as u64)
            .to_be_bytes()
            .to_vec();
        let new_code_hash = CodeHash::from_hash(hash(&new_code));

        let versioned_code = PackageCodeOriginalCodeV1 { code: new_code }.into_versioned();
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
            old_code_hash,
            new_code_hash,
        )
    };

    // Generate the blueprint definition substate updates
    let updated_bp_definition_substate = {
        let versioned_definition: VersionedPackageBlueprintVersionDefinition = reader
            .read_object_collection_entry(
                &tx_processor_pkg_node_id,
                ObjectModuleId::Main,
                ObjectCollectionKey::KeyValue(
                    PackageCollection::BlueprintVersionDefinitionKeyValue.collection_index(),
                    &bp_version_key,
                ),
            )
            .unwrap()
            .unwrap();

        let mut definition = versioned_definition.fully_update_and_into_latest_version();

        for (_, export) in definition.function_exports.iter_mut() {
            export.code_hash = new_code_hash
        }

        scrypto_encode(&definition.into_versioned().into_locked_substate()).unwrap()
    };

    let bp_definition_partition_num = reader
        .get_partition_of_collection(
            &tx_processor_pkg_node_id,
            ObjectModuleId::Main,
            PackageCollection::BlueprintVersionDefinitionKeyValue.collection_index(),
        )
        .unwrap();

    let vm_type_partition_num = reader
        .get_partition_of_collection(
            &tx_processor_pkg_node_id,
            ObjectModuleId::Main,
            PackageCollection::CodeVmTypeKeyValue.collection_index(),
        )
        .unwrap();

    let original_code_partition_num = reader
        .get_partition_of_collection(
            &tx_processor_pkg_node_id,
            ObjectModuleId::Main,
            PackageCollection::CodeOriginalCodeKeyValue.collection_index(),
        )
        .unwrap();

    StateUpdates {
        by_node: indexmap!(
            tx_processor_pkg_node_id => NodeStateUpdates::Delta {
                by_partition: indexmap! {
                    bp_definition_partition_num => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Map(scrypto_encode(&bp_version_key).unwrap()) => DatabaseUpdate::Set(
                                updated_bp_definition_substate
                            )
                        }
                    },
                    vm_type_partition_num => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Map(scrypto_encode(&old_code_hash).unwrap()) => DatabaseUpdate::Delete,
                            SubstateKey::Map(scrypto_encode(&new_code_hash).unwrap()) => DatabaseUpdate::Set(new_vm_type_substate)
                        }
                    },
                    original_code_partition_num => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Map(scrypto_encode(&old_code_hash).unwrap()) => DatabaseUpdate::Delete,
                            SubstateKey::Map(scrypto_encode(&new_code_hash).unwrap()) => DatabaseUpdate::Set(new_code_substate)
                        }
                    },
                }
            }
        ),
    }
}

/// Generates the state updates required for introducing deferred reference check costs
fn generate_ref_check_costs_state_updates() -> StateUpdates {
    StateUpdates::empty().set_substate(
        TRANSACTION_TRACKER,
        BOOT_LOADER_PARTITION,
        BootLoaderField::KernelBoot,
        KernelBoot::V1,
    )
}
