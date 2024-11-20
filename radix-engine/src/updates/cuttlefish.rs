use super::*;
use crate::blueprints::account::*;
use crate::blueprints::consensus_manager::*;
use crate::blueprints::resource::*;
use crate::internal_prelude::*;
use crate::kernel::kernel::KernelBoot;
use crate::object_modules::metadata::*;
use crate::system::system_callback::*;
use crate::system::system_db_reader::*;
use crate::vm::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::identity::*;
use radix_transactions::validation::*;

#[derive(Clone, ScryptoSbor)]
pub struct CuttlefishPart1Settings {
    /// Add configuration for system logic versioning
    pub system_logic_update: UpdateSetting<NoSettings>,
    /// Updating the always visible global nodes to include the account locker package.
    pub kernel_version_update: UpdateSetting<NoSettings>,
    /// Add transaction validation changes
    pub transaction_validation_update: UpdateSetting<NoSettings>,
    /// Adds assert worktop resources methods for the worktop blueprint.
    pub assert_worktop_resources_methods: UpdateSetting<NoSettings>,
    /// Adds getter methods for the account blueprint.
    pub account_getter_methods: UpdateSetting<NoSettings>,
    /// Update the metadata for cuttlefish
    pub update_metadata: UpdateSetting<NoSettings>,
    /// updates the min number of rounds per epoch.
    pub update_number_of_min_rounds_per_epoch:
        UpdateSetting<UpdateNumberOfMinRoundsPerEpochSettings>,
    /// Update identity blueprint to not create a royalty module.
    pub update_identity_to_not_create_royalty_module: UpdateSetting<NoSettings>,
    /// Introduces Crypto Utils v2 with support for
    /// - Blake2b-256,
    /// - Secp256k1 ECDSA verification
    /// - Ed25519 verification
    pub vm_boot_to_enable_crypto_utils_v2: UpdateSetting<NoSettings>,
}

impl UpdateSettings for CuttlefishPart1Settings {
    type UpdateGenerator = CuttlefishPart1Generator;

    fn protocol_version() -> ProtocolVersion {
        ProtocolVersion::CuttlefishPart1
    }

    fn all_enabled_as_default_for_network(network: &NetworkDefinition) -> Self {
        Self {
            system_logic_update: UpdateSetting::enabled_as_default_for_network(network),
            kernel_version_update: UpdateSetting::enabled_as_default_for_network(network),
            transaction_validation_update: UpdateSetting::enabled_as_default_for_network(network),
            assert_worktop_resources_methods: UpdateSetting::enabled_as_default_for_network(
                network,
            ),
            account_getter_methods: UpdateSetting::enabled_as_default_for_network(network),
            update_metadata: UpdateSetting::enabled_as_default_for_network(network),
            update_number_of_min_rounds_per_epoch: UpdateSetting::enabled_as_default_for_network(
                network,
            ),
            update_identity_to_not_create_royalty_module:
                UpdateSetting::enabled_as_default_for_network(network),
            vm_boot_to_enable_crypto_utils_v2: UpdateSetting::enabled_as_default_for_network(
                network,
            ),
        }
    }

    fn all_disabled() -> Self {
        Self {
            system_logic_update: UpdateSetting::Disabled,
            kernel_version_update: UpdateSetting::Disabled,
            transaction_validation_update: UpdateSetting::Disabled,
            assert_worktop_resources_methods: UpdateSetting::Disabled,
            account_getter_methods: UpdateSetting::Disabled,
            update_metadata: UpdateSetting::Disabled,
            update_number_of_min_rounds_per_epoch: UpdateSetting::Disabled,
            update_identity_to_not_create_royalty_module: UpdateSetting::Disabled,
            vm_boot_to_enable_crypto_utils_v2: UpdateSetting::Disabled,
        }
    }

    fn create_generator(&self) -> Self::UpdateGenerator {
        Self::UpdateGenerator {
            settings: self.clone(),
        }
    }
}

/// We had to separate Cuttlefish into two parts, because we noticed an issue
/// after CuttlefishPart1 was deployed to stokenet.
#[derive(Clone, ScryptoSbor)]
pub struct CuttlefishPart2Settings {
    /// Add configuration for system logic versioning
    pub another_system_logic_update: UpdateSetting<NoSettings>,
}

impl UpdateSettings for CuttlefishPart2Settings {
    type UpdateGenerator = CuttlefishPart2Generator;

    fn protocol_version() -> ProtocolVersion {
        ProtocolVersion::CuttlefishPart2
    }

    fn all_enabled_as_default_for_network(network: &NetworkDefinition) -> Self {
        Self {
            another_system_logic_update: UpdateSetting::enabled_as_default_for_network(network),
        }
    }

    fn all_disabled() -> Self {
        Self {
            another_system_logic_update: UpdateSetting::Disabled,
        }
    }

    fn create_generator(&self) -> Self::UpdateGenerator {
        Self::UpdateGenerator {
            settings: self.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, Sbor)]
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

impl UpdateSettingContent for UpdateNumberOfMinRoundsPerEpochSettings {
    fn default_setting(_: &NetworkDefinition) -> Self {
        Self::default()
    }
}

pub struct CuttlefishPart1Generator {
    settings: CuttlefishPart1Settings,
}

impl ProtocolUpdateGenerator for CuttlefishPart1Generator {
    fn batch_groups(&self) -> Vec<Box<dyn ProtocolUpdateBatchGroupGenerator + '_>> {
        vec![FixedBatchGroupGenerator::named("principal")
            .add_batch("primary", |store| {
                generate_part1_batch(store, &self.settings)
            })
            .build()]
    }
}

pub struct CuttlefishPart2Generator {
    settings: CuttlefishPart2Settings,
}

impl ProtocolUpdateGenerator for CuttlefishPart2Generator {
    fn batch_groups(&self) -> Vec<Box<dyn ProtocolUpdateBatchGroupGenerator + '_>> {
        vec![FixedBatchGroupGenerator::named("principal")
            .add_batch("primary", |store| {
                generate_part2_batch(store, &self.settings)
            })
            .build()]
    }
}

#[deny(unused_variables)]
fn generate_part1_batch(
    store: &dyn SubstateDatabase,
    CuttlefishPart1Settings {
        system_logic_update,
        kernel_version_update,
        transaction_validation_update,
        assert_worktop_resources_methods,
        account_getter_methods,
        update_metadata,
        update_number_of_min_rounds_per_epoch,
        update_identity_to_not_create_royalty_module,
        vm_boot_to_enable_crypto_utils_v2,
    }: &CuttlefishPart1Settings,
) -> ProtocolUpdateBatch {
    let mut batch = ProtocolUpdateBatch::empty();

    if let UpdateSetting::Enabled(NoSettings) = &system_logic_update {
        batch.mut_add_flash(
            "cuttlefish-protocol-system-logic-updates",
            generate_system_logic_v2_updates(store),
        );
    }

    if let UpdateSetting::Enabled(NoSettings) = &kernel_version_update {
        batch.mut_add_flash(
            "cuttlefish-protocol-kernel-version-update",
            generate_always_visible_global_nodes_updates(store),
        );
    }

    if let UpdateSetting::Enabled(NoSettings) = &transaction_validation_update {
        batch.mut_add_flash(
            "cuttlefish-transaction-validation-updates",
            generate_cuttlefish_transaction_validation_updates(),
        );
    }
    if let UpdateSetting::Enabled(NoSettings) = &assert_worktop_resources_methods {
        batch.mut_add_flash(
            "cuttlefish-assert-worktop-resources-methods",
            generate_cuttlefish_worktop_methods_extension_state_updates(store),
        );
    }
    if let UpdateSetting::Enabled(NoSettings) = &account_getter_methods {
        batch.mut_add_flash(
            "cuttlefish-account-getter-methods",
            generate_cuttlefish_account_getters_extension_state_updates(store),
        );
    }

    if let UpdateSetting::Enabled(NoSettings) = &update_metadata {
        batch.mut_add_flash(
            "cuttlefish-update-metadata",
            generate_cuttlefish_metadata_fix(store),
        );
    }
    if let UpdateSetting::Enabled(settings) = &update_number_of_min_rounds_per_epoch {
        batch.mut_add_flash(
            "cuttlefish-update-number-of-min-rounds-per-epoch",
            generate_cuttlefish_update_min_rounds_per_epoch(store, *settings),
        );
    }

    if let UpdateSetting::Enabled(NoSettings) = &update_identity_to_not_create_royalty_module {
        batch.mut_add_flash(
            "cuttlefish-update-identity-to-not-create-royalty-module",
            generate_cuttlefish_update_identity_to_not_create_royalty_module(store),
        );
    }

    if let UpdateSetting::Enabled(NoSettings) = &vm_boot_to_enable_crypto_utils_v2 {
        batch.mut_add_flash(
            "cuttlefish-vm-boot-to-enable-crypto-utils-v2",
            generate_vm_boot_to_enable_crypto_utils_v2(),
        );
    }

    batch
}

fn generate_part2_batch(
    store: &dyn SubstateDatabase,
    CuttlefishPart2Settings {
        another_system_logic_update: system_logic_update,
    }: &CuttlefishPart2Settings,
) -> ProtocolUpdateBatch {
    let mut batch = ProtocolUpdateBatch::empty();

    if let UpdateSetting::Enabled(NoSettings) = &system_logic_update {
        batch.mut_add_flash(
            "cuttlefish-part2-protocol-system-logic-updates",
            generate_system_logic_v3_updates(store),
        );
    }

    batch
}

fn generate_system_logic_v2_updates<S: SubstateDatabase + ?Sized>(db: &S) -> StateUpdates {
    let existing_system_boot: SystemBoot = db.get_existing_substate(
        TRANSACTION_TRACKER,
        BOOT_LOADER_PARTITION,
        BootLoaderField::SystemBoot,
    );

    StateUpdates::empty().set_substate(
        TRANSACTION_TRACKER,
        BOOT_LOADER_PARTITION,
        BootLoaderField::SystemBoot,
        SystemBoot::cuttlefish_part1_for_previous_parameters(
            existing_system_boot.into_parameters(),
        ),
    )
}

fn generate_system_logic_v3_updates<S: SubstateDatabase + ?Sized>(db: &S) -> StateUpdates {
    let existing_system_boot: SystemBoot = db.get_existing_substate(
        TRANSACTION_TRACKER,
        BOOT_LOADER_PARTITION,
        BootLoaderField::SystemBoot,
    );

    StateUpdates::empty().set_substate(
        TRANSACTION_TRACKER,
        BOOT_LOADER_PARTITION,
        BootLoaderField::SystemBoot,
        SystemBoot::cuttlefish_part2_for_previous_parameters(
            existing_system_boot.into_parameters(),
        ),
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

fn generate_cuttlefish_worktop_methods_extension_state_updates<S: SubstateDatabase + ?Sized>(
    db: &S,
) -> StateUpdates {
    let (added_functions, schema) = WorktopBlueprintCuttlefishExtension::added_functions_schema();
    let auth_config = WorktopBlueprint::get_definition().auth_config;

    add_public_functions_to_blueprint(
        db,
        RESOURCE_PACKAGE,
        WORKTOP_BLUEPRINT,
        NativeCodeId::ResourceCode2,
        added_functions,
        schema,
        auth_config,
    )
}

fn generate_cuttlefish_account_getters_extension_state_updates<S: SubstateDatabase + ?Sized>(
    db: &S,
) -> StateUpdates {
    let (added_functions, schema) = AccountBlueprintCuttlefishExtension::added_functions_schema();

    // Update the auth config of the account blueprint to have these added methods and have them be
    // public.
    let auth_config = {
        let mut auth_config = AccountBlueprint::get_definition().auth_config;
        let MethodAuthTemplate::StaticRoleDefinition(StaticRoleDefinition {
            ref mut methods, ..
        }) = auth_config.method_auth
        else {
            panic!("Doesn't have a static role definition")
        };
        methods.extend(
            added_functions
                .keys()
                .map(ToOwned::to_owned)
                .map(|ident| MethodKey { ident })
                .map(|key| (key, MethodAccessibility::Public)),
        );
        auth_config
    };

    add_public_functions_to_blueprint(
        db,
        ACCOUNT_PACKAGE,
        ACCOUNT_BLUEPRINT,
        NativeCodeId::AccountCode3,
        added_functions,
        schema,
        auth_config,
    )
}

fn generate_cuttlefish_metadata_fix<S: SubstateDatabase + ?Sized>(db: &S) -> StateUpdates {
    struct MetadataUpdates {
        pub name: Option<MetadataValue>,
        pub description: Option<MetadataValue>,
        pub icon_url: Option<MetadataValue>,
    }

    impl MetadataUpdates {
        pub fn into_map(self) -> IndexMap<String, MetadataValue> {
            [
                self.name.map(|value| ("name", value)),
                self.description.map(|value| ("description", value)),
                self.icon_url.map(|value| ("icon_url", value)),
            ]
            .into_iter()
            .flatten()
            .map(|(k, v)| (k.to_owned(), v))
            .collect()
        }
    }

    let reader = SystemDatabaseReader::new(db);

    // The metadata entries that we would like to update and the values that we would like to update
    // them to be.
    let metadata_updates = indexmap! {
        XRD => MetadataUpdates {
            name: None,
            description: None,
            icon_url: Some(MetadataValue::Url(UncheckedUrl("https://assets.radixdlt.com/icons/icon-xrd.png".into())))
        },
        PACKAGE_OF_DIRECT_CALLER_RESOURCE => MetadataUpdates {
            name: Some(MetadataValue::String("Package of Direct Caller Resource".into())),
            description: Some(MetadataValue::String("This is an implicit proof resource, intended for verifying access by specific code. See the info_url for further information.".into())),
            icon_url: Some(MetadataValue::Url(UncheckedUrl("https://assets.radixdlt.com/icons/icon-package_of_direct_caller_resource.png".into())))
        },
        GLOBAL_CALLER_RESOURCE => MetadataUpdates {
            name: Some(MetadataValue::String("Global Caller Resource".into())),
            description: Some(MetadataValue::String("This is an implicit proof resource, intended for verifying access by a specific global caller. In cases where you wish to find out the global caller, you can require the caller to pass their claimed global address into the method, and then verify it with this rule. See the info_url for further information.".into())),
            icon_url: Some(MetadataValue::Url(UncheckedUrl("https://assets.radixdlt.com/icons/icon-global_caller_resource.png".into())))
        },
        SECP256K1_SIGNATURE_RESOURCE => MetadataUpdates {
            name: Some(MetadataValue::String("ECDSA Secp256k1 Signature Resource".into())),
            description: Some(MetadataValue::String("This is an implicit proof resource, intended for verifying access by a manifest signed with the given ECDSA Secp256k1 key hash. See the info_url for further information.".into())),
            icon_url: Some(MetadataValue::Url(UncheckedUrl("https://assets.radixdlt.com/icons/icon-ecdsa_secp256k1_signature_resource.png".into())))
        },
        ED25519_SIGNATURE_RESOURCE => MetadataUpdates {
            name: Some(MetadataValue::String("EdDSA Ed25519 Signature Resource".into())),
            description: Some(MetadataValue::String("This is an implicit proof resource, intended for verifying access by a manifest signed with the given EdDSA Ed25519 key hash. See the info_url for further information.".into())),
            icon_url: Some(MetadataValue::Url(UncheckedUrl("https://assets.radixdlt.com/icons/icon-eddsa_ed25519_signature_resource.png".into())))
        },
        SYSTEM_EXECUTION_RESOURCE => MetadataUpdates {
            name: Some(MetadataValue::String("System Execution Resource".into())),
            description: Some(MetadataValue::String("This is an implicit proof resource, intended for verifying access by a manifest of a certain type of system transaction, such as a protocol update or a validator transaction. See the info_url for further information.".into())),
            icon_url: Some(MetadataValue::Url(UncheckedUrl("https://assets.radixdlt.com/icons/icon-system_execution_resource.png".into())))
        },
    };

    // We would like to add an `info_url` entry for the various entities that we have. The this is
    // the mapping that we're using.
    let info_url_metadata = [
        (XRD.into_node_id(), "https://www.radixdlt.com/info-url/xrd"),
        (
            SECP256K1_SIGNATURE_RESOURCE.into_node_id(),
            "https://www.radixdlt.com/info-url/secp256k1-signature-resource",
        ),
        (
            ED25519_SIGNATURE_RESOURCE.into_node_id(),
            "https://www.radixdlt.com/info-url/ed25519-signature-resource",
        ),
        (
            PACKAGE_OF_DIRECT_CALLER_RESOURCE.into_node_id(),
            "https://www.radixdlt.com/info-url/package-of-direct-caller-resource",
        ),
        (
            GLOBAL_CALLER_RESOURCE.into_node_id(),
            "https://www.radixdlt.com/info-url/global-caller-resource",
        ),
        (
            SYSTEM_EXECUTION_RESOURCE.into_node_id(),
            "https://www.radixdlt.com/info-url/system-execution-resource",
        ),
        (
            PACKAGE_OWNER_BADGE.into_node_id(),
            "https://www.radixdlt.com/info-url/package-owner-badge",
        ),
        (
            VALIDATOR_OWNER_BADGE.into_node_id(),
            "https://www.radixdlt.com/info-url/validator-owner-badge",
        ),
        (
            ACCOUNT_OWNER_BADGE.into_node_id(),
            "https://www.radixdlt.com/info-url/account-owner-badge",
        ),
        (
            IDENTITY_OWNER_BADGE.into_node_id(),
            "https://www.radixdlt.com/info-url/identity-owner-badge",
        ),
        (
            PACKAGE_PACKAGE.into_node_id(),
            "https://www.radixdlt.com/info-url/package-package",
        ),
        (
            RESOURCE_PACKAGE.into_node_id(),
            "https://www.radixdlt.com/info-url/resource-package",
        ),
        (
            ACCOUNT_PACKAGE.into_node_id(),
            "https://www.radixdlt.com/info-url/account-package",
        ),
        (
            IDENTITY_PACKAGE.into_node_id(),
            "https://www.radixdlt.com/info-url/identity-package",
        ),
        (
            CONSENSUS_MANAGER_PACKAGE.into_node_id(),
            "https://www.radixdlt.com/info-url/consensus-manager-package",
        ),
        (
            ACCESS_CONTROLLER_PACKAGE.into_node_id(),
            "https://www.radixdlt.com/info-url/access-controller-package",
        ),
        (
            POOL_PACKAGE.into_node_id(),
            "https://www.radixdlt.com/info-url/pool-package",
        ),
        (
            TRANSACTION_PROCESSOR_PACKAGE.into_node_id(),
            "https://www.radixdlt.com/info-url/transaction-processor-package",
        ),
        (
            METADATA_MODULE_PACKAGE.into_node_id(),
            "https://www.radixdlt.com/info-url/metadata-module-package",
        ),
        (
            ROYALTY_MODULE_PACKAGE.into_node_id(),
            "https://www.radixdlt.com/info-url/royalty-module-package",
        ),
        (
            ROLE_ASSIGNMENT_MODULE_PACKAGE.into_node_id(),
            "https://www.radixdlt.com/info-url/role-assignment-module-package",
        ),
        (
            TEST_UTILS_PACKAGE.into_node_id(),
            "https://www.radixdlt.com/info-url/test-utils-package",
        ),
        (
            GENESIS_HELPER_PACKAGE.into_node_id(),
            "https://www.radixdlt.com/info-url/genesis-helper-package",
        ),
        (
            FAUCET_PACKAGE.into_node_id(),
            "https://www.radixdlt.com/info-url/faucet-package",
        ),
        (
            TRANSACTION_TRACKER_PACKAGE.into_node_id(),
            "https://www.radixdlt.com/info-url/transaction-tracker-package",
        ),
        (
            LOCKER_PACKAGE.into_node_id(),
            "https://www.radixdlt.com/info-url/locker-package",
        ),
        (
            CONSENSUS_MANAGER.into_node_id(),
            "https://www.radixdlt.com/info-url/consensus-manager",
        ),
        (
            GENESIS_HELPER.into_node_id(),
            "https://www.radixdlt.com/info-url/genesis-helper",
        ),
        (
            FAUCET.into_node_id(),
            "https://www.radixdlt.com/info-url/faucet",
        ),
        (
            TRANSACTION_TRACKER.into_node_id(),
            "https://www.radixdlt.com/info-url/transaction-tracker",
        ),
    ];

    let mut state_updates = StateUpdates::empty();
    for (resource_address, metadata_updates) in metadata_updates.into_iter() {
        for (key, value) in metadata_updates.into_map().into_iter() {
            let partition_number = reader
                .get_partition_of_collection(
                    resource_address.as_node_id(),
                    ModuleId::Metadata,
                    MetadataCollection::EntryKeyValue.collection_index(),
                )
                .unwrap();

            state_updates = state_updates.set_substate(
                resource_address,
                partition_number,
                SubstateKey::Map(
                    scrypto_encode(&MetadataEntryKeyPayload { content: key }).unwrap(),
                ),
                value.into_locked_substate(),
            );
        }
    }
    for (node_id, info_url) in info_url_metadata.into_iter() {
        let partition_number = reader
            .get_partition_of_collection(
                &node_id,
                ModuleId::Metadata,
                MetadataCollection::EntryKeyValue.collection_index(),
            )
            .unwrap();

        state_updates = state_updates.set_substate(
            node_id,
            partition_number,
            SubstateKey::Map(
                scrypto_encode(&MetadataEntryKeyPayload {
                    content: "info_url".to_owned(),
                })
                .unwrap(),
            ),
            MetadataValue::Url(UncheckedUrl(info_url.into())).into_locked_substate(),
        );
    }

    state_updates
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

fn generate_cuttlefish_update_identity_to_not_create_royalty_module<
    S: SubstateDatabase + ?Sized,
>(
    db: &S,
) -> StateUpdates {
    let reader = SystemDatabaseReader::new(db);
    let node_id = IDENTITY_PACKAGE.into_node_id();
    let blueprint_version_key = BlueprintVersionKey {
        blueprint: IDENTITY_BLUEPRINT.to_string(),
        version: Default::default(),
    };

    // Create substates for the new code.
    let (code_hash, (code_substate, vm_type_substate)) = {
        let original_code = (NativeCodeId::IdentityCode2 as u64).to_be_bytes().to_vec();

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

    // Update the definition of the existing blueprint so that new code is used
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

        for function_name in [IDENTITY_CREATE_ADVANCED_IDENT, IDENTITY_CREATE_IDENT] {
            blueprint_definition
                .function_exports
                .get_mut(function_name)
                .expect("This function must exist")
                .code_hash = code_hash;
        }

        blueprint_definition
            .hook_exports
            .get_mut(&BlueprintHook::OnVirtualize)
            .expect("Identity::OnVirtualize hook must exist")
            .code_hash = code_hash;

        blueprint_definition.into_locked_substate()
    };

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

    // Generate state updates
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
            ),
    )
}

fn generate_vm_boot_to_enable_crypto_utils_v2() -> StateUpdates {
    StateUpdates::empty().set_substate(
        TRANSACTION_TRACKER,
        BOOT_LOADER_PARTITION,
        BootLoaderField::VmBoot,
        VmBoot::V1 {
            scrypto_version: ScryptoVmVersion::crypto_utils_v2().into(),
        },
    )
}

fn add_public_functions_to_blueprint<S: SubstateDatabase + ?Sized>(
    db: &S,
    package: PackageAddress,
    blueprint: &str,
    native_code_id: NativeCodeId,
    added_functions: IndexMap<String, FunctionSchemaInit>,
    schema: VersionedSchema<ScryptoCustomSchema>,
    auth_config: AuthConfig,
) -> StateUpdates {
    let reader = SystemDatabaseReader::new(db);
    let node_id = package.into_node_id();
    let blueprint_version_key = BlueprintVersionKey {
        blueprint: blueprint.to_string(),
        version: Default::default(),
    };

    // Creating the original code substates for extension.
    let (code_hash, (code_substate, vm_type_substate)) = {
        let original_code = (native_code_id as u64).to_be_bytes().to_vec();

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
                auth_config.into_locked_substate(),
            ),
    )
}
