use radix_engine_interface::blueprints::account::*;
use radix_transactions::validation::*;

use super::*;
use crate::blueprints::account::*;
use crate::blueprints::consensus_manager::*;
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
    /// updates the min number of rounds per epoch.
    pub update_number_of_min_rounds_per_epoch:
        UpdateSetting<UpdateNumberOfMinRoundsPerEpochSettings>,
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
            update_number_of_min_rounds_per_epoch: UpdateSetting::enabled_as_default_for_network(
                network,
            ),
        }
    }

    fn all_disabled() -> Self {
        Self {
            system_logic_update: UpdateSetting::Disabled,
            kernel_version_update: UpdateSetting::Disabled,
            transaction_validation_update: UpdateSetting::Disabled,
            account_getter_methods: UpdateSetting::Disabled,
            update_number_of_min_rounds_per_epoch: UpdateSetting::Disabled,
        }
    }

    fn create_batch_generator(&self) -> Self::BatchGenerator {
        Self::BatchGenerator {
            settings: self.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum UpdateNumberOfMinRoundsPerEpochSettings {
    Set { value: u64 },
    SetIfEquals { if_equals: u64, to_value: u64 },
}

impl Default for UpdateNumberOfMinRoundsPerEpochSettings {
    fn default() -> Self {
        Self::SetIfEquals {
            if_equals: 500,
            to_value: 100,
        }
    }
}

impl UpdateSettingMarker for UpdateNumberOfMinRoundsPerEpochSettings {}

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
        update_number_of_min_rounds_per_epoch,
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
    if let UpdateSetting::Enabled(settings) = &update_number_of_min_rounds_per_epoch {
        transactions.push(ProtocolUpdateTransactionDetails::flash(
            "cuttlefish-update-number-of-min-rounds-per-epoch",
            generate_cuttlefish_update_min_rounds_per_epoch(store, *settings),
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
    let blueprint_version_definition_partition_number =
        PackagePartitionOffset::BlueprintVersionDefinitionKeyValue.as_main_partition();
    let code_vm_type_partition_number =
        PackagePartitionOffset::CodeVmTypeKeyValue.as_main_partition();
    let code_original_code_partition_number =
        PackagePartitionOffset::CodeOriginalCodeKeyValue.as_main_partition();
    let schema_partition_number = SCHEMAS_PARTITION.at_offset(PartitionOffset(0)).unwrap();
    let blueprint_version_auth_config_partition_number =
        PackagePartitionOffset::BlueprintVersionAuthConfigKeyValue.as_main_partition();

    // Generating the state updates
    StateUpdates::empty().set_node_updates(
        node_id,
        NodeStateUpdates::empty()
            .set_substate(
                blueprint_version_definition_partition_number,
                SubstateKey::map(&blueprint_version_key),
                blueprint_definition_substate,
            )
            .set_substate(
                code_vm_type_partition_number,
                SubstateKey::map(&code_hash),
                vm_type_substate,
            )
            .set_substate(
                code_original_code_partition_number,
                SubstateKey::map(&code_hash),
                code_substate,
            )
            .set_substate(
                schema_partition_number,
                SubstateKey::map(&schema_hash),
                schema_substate,
            )
            .set_substate(
                blueprint_version_auth_config_partition_number,
                SubstateKey::map(&blueprint_version_key),
                auth_config,
            ),
    )
}

fn generate_cuttlefish_update_min_rounds_per_epoch<S: SubstateDatabase + ?Sized>(
    db: &S,
    settings: UpdateNumberOfMinRoundsPerEpochSettings,
) -> StateUpdates {
    let mut consensus_manager_config = db
        .get_existing_substate::<FieldSubstate<VersionedConsensusManagerConfiguration>>(
            CONSENSUS_MANAGER,
            MAIN_BASE_PARTITION,
            ConsensusManagerField::Configuration,
        )
        .into_payload()
        .fully_update_and_into_latest_version();
    let min_rounds_per_epoch = &mut consensus_manager_config
        .config
        .epoch_change_condition
        .min_round_count;

    match settings {
        UpdateNumberOfMinRoundsPerEpochSettings::Set { value } => *min_rounds_per_epoch = value,
        UpdateNumberOfMinRoundsPerEpochSettings::SetIfEquals {
            if_equals,
            to_value,
        } => {
            if *min_rounds_per_epoch == if_equals {
                *min_rounds_per_epoch = to_value
            }
        }
    }

    StateUpdates::empty().set_substate(
        CONSENSUS_MANAGER,
        MAIN_BASE_PARTITION,
        ConsensusManagerField::Configuration,
        consensus_manager_config.into_locked_substate(),
    )
}
