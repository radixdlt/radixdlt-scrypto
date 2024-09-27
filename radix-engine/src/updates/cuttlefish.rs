use radix_transactions::validation::*;

use super::*;
use crate::kernel::kernel::KernelBoot;
use crate::object_modules::metadata::*;
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
    /// Update the metadata for cuttlefish
    pub update_metadata: UpdateSetting<NoSettings>,
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
            update_metadata: UpdateSetting::enabled_as_default_for_network(network),
        }
    }

    fn all_disabled() -> Self {
        Self {
            system_logic_update: UpdateSetting::Disabled,
            kernel_version_update: UpdateSetting::Disabled,
            transaction_validation_update: UpdateSetting::Disabled,
            update_metadata: UpdateSetting::Disabled,
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
        kernel_version_update,
        transaction_validation_update,
        update_metadata,
    }: &CuttlefishSettings,
) -> ProtocolUpdateBatch {
    let mut transactions = vec![];
    if let UpdateSetting::Enabled(NoSettings) = &system_logic_update {
        transactions.push(ProtocolUpdateTransactionDetails::flash(
            "cuttlefish-protocol-system-logic-updates",
            generate_system_logic_v2_updates(store),
        ));
    }
    if let UpdateSetting::Enabled(_settings) = &kernel_version_update {
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
    if let UpdateSetting::Enabled(NoSettings) = &update_metadata {
        transactions.push(ProtocolUpdateTransactionDetails::flash(
            "cuttlefish-update-metadata",
            generate_cuttlefish_metadata_fix(store),
        ));
    }
    ProtocolUpdateBatch { transactions }
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
            name: Some(MetadataValue::String("Package Caller Resource".into())),
            description: Some(MetadataValue::String("Badges generated automatically by the Radix system to represent the authority of the package for a direct caller. These badges cease to exist at the end of their transaction.".into())),
            icon_url: Some(MetadataValue::Url(UncheckedUrl("https://assets.radixdlt.com/icons/icon-package_of_direct_caller_resource.png".into())))
        },
        GLOBAL_CALLER_RESOURCE => MetadataUpdates {
            name: Some(MetadataValue::String("Global Caller Resource".into())),
            description: Some(MetadataValue::String("Badges generated automatically by the Radix system to represent the authority of a global caller. These badges cease to exist at the end of their transaction.".into())),
            icon_url: Some(MetadataValue::Url(UncheckedUrl("https://assets.radixdlt.com/icons/icon-global_caller_resource.png".into())))
        },
        SECP256K1_SIGNATURE_RESOURCE => MetadataUpdates {
            name: Some(MetadataValue::String("ECDSA Secp256k1 Signature Resource".into())),
            description: Some(MetadataValue::String("Badges generated automatically by the Radix system to represent ECDSA Secp256k1 signatures applied to transactions. These badges cease to exist at the end of their transaction.".into())),
            icon_url: Some(MetadataValue::Url(UncheckedUrl("https://assets.radixdlt.com/icons/icon-ecdsa_secp256k1_signature_resource.png".into())))
        },
        ED25519_SIGNATURE_RESOURCE => MetadataUpdates {
            name: Some(MetadataValue::String("EdDSA Ed25519 Resource".into())),
            description: Some(MetadataValue::String("Badges generated automatically by the Radix system to represent EdDSA Ed25519 signatures applied to transactions. These badges cease to exist at the end of their transaction.".into())),
            icon_url: Some(MetadataValue::Url(UncheckedUrl("https://assets.radixdlt.com/icons/icon-eddsa_ed25519_signature_resource.png".into())))
        },
        SYSTEM_EXECUTION_RESOURCE => MetadataUpdates {
            name: Some(MetadataValue::String("System Execution Resource".into())),
            description: Some(MetadataValue::String("Badges are created under this resource to represent the Radix system's authority at genesis and to affect changes to system entities during protocol updates, or to represent the Radix system's authority in the regularly occurring system transactions including round and epoch changes.".into())),
            icon_url: Some(MetadataValue::Url(UncheckedUrl("https://assets.radixdlt.com/icons/icon-system_execution_resource.png".into())))
        },
    };

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

    state_updates
}
