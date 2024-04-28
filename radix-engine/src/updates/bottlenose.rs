use super::*;
use crate::blueprints::locker::LockerNativePackage;
use crate::blueprints::models::KeyValueEntryContentSource;
use crate::blueprints::package::*;
use crate::object_modules::role_assignment::*;
use crate::system::system_callback::*;
use crate::system::system_db_reader::*;
use crate::track::*;
use crate::transaction::*;
use crate::vm::*;
use radix_engine_interface::blueprints::account::*;

#[derive(Clone)]
pub struct BottlenoseSettings {
    /// Exposes a getter method for reading owner role rule.
    pub add_owner_role_getter: UpdateSetting<()>,

    /// Various system patches.
    pub add_system_patches: UpdateSetting<()>,

    /// Introduces the account locker blueprint.
    pub add_locker_package: UpdateSetting<()>,

    /// Makes some behavioral changes to the try_deposit_or_refund (and batch variants too) method
    /// on the account blueprint.
    pub fix_account_try_deposit_or_refund_behaviour: UpdateSetting<()>,

    /// Moves various protocol parameters to state.
    pub move_protocol_params_to_state: UpdateSetting<ProtocolParamsSettings>,
}

#[derive(Clone)]
pub struct ProtocolParamsSettings {
    pub network_definition: NetworkDefinition,
}

impl DefaultForNetwork for ProtocolParamsSettings {
    fn default_for_network(network_definition: &NetworkDefinition) -> Self {
        Self {
            network_definition: network_definition.clone(),
        }
    }
}

impl UpdateSettings for BottlenoseSettings {
    type BatchGenerator = BottlenoseBatchGenerator;

    fn all_enabled_as_default_for_network(network: &NetworkDefinition) -> Self {
        Self {
            add_owner_role_getter: UpdateSetting::enabled_as_default_for_network(network),
            add_system_patches: UpdateSetting::enabled_as_default_for_network(network),
            add_locker_package: UpdateSetting::enabled_as_default_for_network(network),
            move_protocol_params_to_state: UpdateSetting::enabled_as_default_for_network(network),
            fix_account_try_deposit_or_refund_behaviour:
                UpdateSetting::enabled_as_default_for_network(network),
        }
    }

    fn all_disabled() -> Self {
        Self {
            add_owner_role_getter: UpdateSetting::Disabled,
            add_system_patches: UpdateSetting::Disabled,
            add_locker_package: UpdateSetting::Disabled,
            move_protocol_params_to_state: UpdateSetting::Disabled,
            fix_account_try_deposit_or_refund_behaviour: UpdateSetting::Disabled,
        }
    }

    fn create_batch_generator(&self) -> Self::BatchGenerator {
        BottlenoseBatchGenerator {
            settings: self.clone(),
        }
    }
}

#[derive(Clone)]
pub struct BottlenoseBatchGenerator {
    settings: BottlenoseSettings,
}

impl ProtocolUpdateBatchGenerator for BottlenoseBatchGenerator {
    fn generate_batch(
        &self,
        store: &dyn SubstateDatabase,
        batch_index: u32,
    ) -> Option<ProtocolUpdateBatch> {
        match batch_index {
            // Just a single batch for Bottlenose, perhaps in future updates we should have separate batches for each update?
            0 => Some(generate_principal_batch(store, &self.settings)),
            _ => None,
        }
    }
}

fn generate_principal_batch(
    store: &dyn SubstateDatabase,
    settings: &BottlenoseSettings,
) -> ProtocolUpdateBatch {
    let mut transactions = vec![];
    if let UpdateSetting::Enabled(_) = &settings.add_owner_role_getter {
        transactions.push(ProtocolUpdateTransactionDetails::flash(
            "bottlenose-owner-role-getter",
            generate_owner_role_getter_state_updates(store),
        ));
    }
    if let UpdateSetting::Enabled(_) = &settings.add_system_patches {
        transactions.push(ProtocolUpdateTransactionDetails::flash(
            "bottlenose-system-patches",
            generate_system_patches(),
        ));
    }
    if let UpdateSetting::Enabled(_) = &settings.add_locker_package {
        transactions.push(ProtocolUpdateTransactionDetails::flash(
            "bottlenose-locker-package",
            generate_locker_package_state_updates(),
        ));
    }
    if let UpdateSetting::Enabled(_) = &settings.fix_account_try_deposit_or_refund_behaviour {
        transactions.push(ProtocolUpdateTransactionDetails::flash(
            "bottlenose-account-try-deposit-or-refund",
            generate_account_bottlenose_extension_state_updates(store),
        ));
    }
    if let UpdateSetting::Enabled(settings) = &settings.move_protocol_params_to_state {
        transactions.push(ProtocolUpdateTransactionDetails::flash(
            "bottlenose-protocol-params-to-state",
            generate_protocol_params_to_state_updates(settings.network_definition.clone()),
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

pub fn generate_owner_role_getter_state_updates<S: SubstateDatabase + ?Sized>(
    db: &S,
) -> StateUpdates {
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

pub fn generate_system_patches() -> StateUpdates {
    // TODO
    StateUpdates::default()
}

pub fn generate_locker_package_state_updates() -> StateUpdates {
    let package_definition = LockerNativePackage::definition();
    let package_structure = PackageNativePackage::validate_and_build_package_structure(
        package_definition,
        VmType::Native,
        (NativeCodeId::LockerCode1 as u64).to_be_bytes().to_vec(),
        Default::default(),
        &VmVersion::latest(),
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

pub fn generate_account_bottlenose_extension_state_updates<S: SubstateDatabase + ?Sized>(
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

pub fn generate_protocol_params_to_state_updates(
    network_definition: NetworkDefinition,
) -> StateUpdates {
    StateUpdates {
        by_node: indexmap!(
            TRANSACTION_TRACKER.into_node_id() => NodeStateUpdates::Delta {
                by_partition: indexmap! {
                    BOOT_LOADER_PARTITION => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Field(BOOT_LOADER_SYSTEM_SUBSTATE_FIELD_KEY) => DatabaseUpdate::Set(
                                scrypto_encode(&SystemBoot::V1(SystemParameters {
                                    network_definition,
                                    costing_parameters: CostingParameters::babylon_genesis(),
                                    limit_parameters: LimitParameters::babylon_genesis(),
                                    max_per_function_royalty_in_xrd: Decimal::try_from(MAX_PER_FUNCTION_ROYALTY_IN_XRD).unwrap(),
                                })).unwrap()
                            )
                        }
                    },
                }
            }
        ),
    }
}
