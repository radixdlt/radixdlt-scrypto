use radix_engine_interface::blueprints::account::*;
use radix_transactions::validation::*;

use super::*;
use crate::blueprints::account::*;
use crate::kernel::kernel::KernelBoot;
use crate::system::system_callback::*;
use crate::system::system_db_reader::*;

#[derive(Clone)]
pub struct CuttlefishSettings {
    /// Add configuration for system logic versioning
    pub system_logic_update: UpdateSetting<NoSettings>,
    /// Updating the always visible global nodes to include the account locker package.
    pub kernel_version_update: UpdateSetting<NoSettings>,
    /// Add transaction validation changes
    pub transaction_validation_update: UpdateSetting<NoSettings>,
    /// Adds getter methods for the account blueprint.
    pub account_getter_methods: UpdateSetting<NoSettings>,
}

impl UpdateSettings for CuttlefishSettings {
    type BatchGenerator = CuttlefishBatchGenerator;

    fn protocol_version() -> ProtocolVersion {
        ProtocolVersion::Cuttlefish
    }

    fn all_enabled_as_default_for_network(network: &NetworkDefinition) -> Self {
        Self {
            system_logic_update: UpdateSetting::enabled_as_default_for_network(network),
            kernel_version_update: UpdateSetting::enabled_as_default_for_network(network),
            transaction_validation_update: UpdateSetting::enabled_as_default_for_network(network),
            account_getter_methods: UpdateSetting::enabled_as_default_for_network(network),
        }
    }

    fn all_disabled() -> Self {
        Self {
            system_logic_update: UpdateSetting::Disabled,
            kernel_version_update: UpdateSetting::Disabled,
            transaction_validation_update: UpdateSetting::Disabled,
            account_getter_methods: UpdateSetting::Disabled,
        }
    }

    fn create_batch_generator(&self) -> Self::BatchGenerator {
        Self::BatchGenerator {
            settings: self.clone(),
        }
    }
}

#[derive(Clone)]
pub struct CuttlefishBatchGenerator {
    settings: CuttlefishSettings,
}

impl ProtocolUpdateBatchGenerator for CuttlefishBatchGenerator {
    fn generate_batch(
        &self,
        store: &dyn SubstateDatabase,
        batch_group_index: usize,
        batch_index: usize,
    ) -> ProtocolUpdateBatch {
        match (batch_group_index, batch_index) {
            // Each batch is committed as one.
            // To avoid large memory usage, large batches should be split up,
            // e.g. `(0, 1) => generate_second_batch(..)`
            (0, 0) => generate_principal_batch(store, &self.settings),
            _ => {
                panic!("batch index out of range")
            }
        }
    }

    fn batch_count(&self, batch_group_index: usize) -> usize {
        match batch_group_index {
            0 => 1,
            _ => panic!("Invalid batch_group_index: {batch_group_index}"),
        }
    }

    fn batch_group_descriptors(&self) -> Vec<String> {
        vec!["Principal".to_string()]
    }
}

#[deny(unused_variables)]
fn generate_principal_batch(
    store: &dyn SubstateDatabase,
    CuttlefishSettings {
        system_logic_update,
        kernel_version_update: always_visible_global_nodes_update,
        transaction_validation_update,
        account_getter_methods,
    }: &CuttlefishSettings,
) -> ProtocolUpdateBatch {
    let mut transactions = vec![];
    if let UpdateSetting::Enabled(NoSettings) = &system_logic_update {
        transactions.push(ProtocolUpdateTransactionDetails::flash(
            "cuttlefish-protocol-system-logic-updates",
            generate_system_logic_v2_updates(store),
        ));
    }
    if let UpdateSetting::Enabled(_settings) = &always_visible_global_nodes_update {
        transactions.push(ProtocolUpdateTransactionDetails::flash(
            "cuttlefish-protocol-kernel-version-update",
            generate_always_visible_global_nodes_updates(store),
        ));
    }
    if let UpdateSetting::Enabled(NoSettings) = &transaction_validation_update {
        transactions.push(ProtocolUpdateTransactionDetails::flash(
            "cuttlefish-transaction-validation-updates",
            generate_cuttlefish_transaction_validation_updates(),
        ));
    }
    if let UpdateSetting::Enabled(NoSettings) = &account_getter_methods {
        transactions.push(ProtocolUpdateTransactionDetails::flash(
            "cuttlefish-account-getter-methods",
            generate_cuttlefish_account_getters_extension_state_updates(store),
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

fn generate_system_logic_v2_updates<S: SubstateDatabase + ?Sized>(db: &S) -> StateUpdates {
    let system_boot: SystemBoot = db.get_existing_substate(
        TRANSACTION_TRACKER,
        BOOT_LOADER_PARTITION,
        BootLoaderField::SystemBoot,
    );

    let cur_system_parameters = match system_boot {
        SystemBoot::V1(parameters) => parameters,
        _ => panic!("Unexpected SystemBoot version"),
    };

    StateUpdates::empty().set_substate(
        TRANSACTION_TRACKER,
        BOOT_LOADER_PARTITION,
        BootLoaderField::SystemBoot,
        SystemBoot::cuttlefish_for_previous_parameters(cur_system_parameters),
    )
}

fn generate_always_visible_global_nodes_updates<S: SubstateDatabase + ?Sized>(
    db: &S,
) -> StateUpdates {
    let KernelBoot::V1 = db.get_existing_substate::<KernelBoot>(
        TRANSACTION_TRACKER,
        BOOT_LOADER_PARTITION,
        BootLoaderField::KernelBoot,
    ) else {
        panic!("Unexpected KernelBoot version")
    };

    StateUpdates::empty().set_substate(
        TRANSACTION_TRACKER,
        BOOT_LOADER_PARTITION,
        BootLoaderField::KernelBoot,
        KernelBoot::cuttlefish(),
    )
}

fn generate_cuttlefish_transaction_validation_updates() -> StateUpdates {
    StateUpdates::empty().set_substate(
        TRANSACTION_TRACKER,
        BOOT_LOADER_PARTITION,
        BootLoaderField::TransactionValidationConfiguration,
        TransactionValidationConfigurationSubstate::new(
            TransactionValidationConfigurationVersions::V1(
                TransactionValidationConfigV1::cuttlefish(),
            ),
        ),
    )
}

fn generate_cuttlefish_account_getters_extension_state_updates<S: SubstateDatabase + ?Sized>(
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
        let original_code = (NativeCodeId::AccountCode3 as u64).to_be_bytes().to_vec();

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
    let (added_functions, schema) = AccountBlueprintCuttlefishExtension::added_functions_schema();
    let (schema_hash, schema_substate) =
        (schema.generate_schema_hash(), schema.into_locked_substate());

    // Update the auth config of the account blueprint to have these added methods and have them be
    // public.
    let auth_config = {
        let mut auth_config = AccountBlueprint::get_definition().auth_config;
        let MethodAuthTemplate::StaticRoleDefinition(StaticRoleDefinition {
            ref mut methods, ..
        }) = auth_config.method_auth
        else {
            panic!("Account doesn't have a static role definition")
        };
        methods.extend(
            added_functions
                .keys()
                .map(ToOwned::to_owned)
                .map(|ident| MethodKey { ident })
                .map(|key| (key, MethodAccessibility::Public)),
        );
        auth_config.into_locked_substate()
    };

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
    let [blueprint_version_definition_partition_number, code_vm_type_partition_number, code_original_code_partition_number, schema_partition_number, blueprint_version_auth_config_partition_number] =
        [
            PackageCollection::BlueprintVersionDefinitionKeyValue,
            PackageCollection::CodeVmTypeKeyValue,
            PackageCollection::CodeOriginalCodeKeyValue,
            PackageCollection::SchemaKeyValue,
            PackageCollection::BlueprintVersionAuthConfigKeyValue,
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
                    },
                    blueprint_version_auth_config_partition_number => PartitionStateUpdates::Delta {
                        by_substate: indexmap! {
                            SubstateKey::Map(scrypto_encode!(&blueprint_version_key)) => DatabaseUpdate::Set(
                                scrypto_encode!(&auth_config)
                            )
                        }
                    },
                }
            }
        ),
    }
}
