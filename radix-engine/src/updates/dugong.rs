use super::*;
use crate::internal_prelude::*;

#[derive(Clone, ScryptoSbor)]
pub struct DugongSettings {
    pub native_entity_metadata_updates: UpdateSetting<NoSettings>,
    /// Enables WASM proposals: reference-types and multi-value
    pub vm_boot_wasm_new_features: UpdateSetting<NoSettings>,
}

impl UpdateSettings for DugongSettings {
    type UpdateGenerator = DugongGenerator;

    fn protocol_version() -> ProtocolVersion {
        ProtocolVersion::Dugong
    }

    fn all_enabled_as_default_for_network(network: &NetworkDefinition) -> Self {
        Self {
            native_entity_metadata_updates: UpdateSetting::enabled_as_default_for_network(network),
            vm_boot_wasm_new_features: UpdateSetting::enabled_as_default_for_network(network),
        }
    }

    fn all_disabled() -> Self {
        Self {
            native_entity_metadata_updates: UpdateSetting::Disabled,
            vm_boot_wasm_new_features: UpdateSetting::Disabled,
        }
    }

    fn create_generator(&self) -> Self::UpdateGenerator {
        Self::UpdateGenerator {
            settings: self.clone(),
        }
    }
}

pub struct DugongGenerator {
    settings: DugongSettings,
}

impl ProtocolUpdateGenerator for DugongGenerator {
    fn batch_groups(&self) -> Vec<Box<dyn ProtocolUpdateBatchGroupGenerator + '_>> {
        vec![FixedBatchGroupGenerator::named("principal")
            .add_batch("primary", |store| {
                generate_main_batch(store, &self.settings)
            })
            .build()]
    }
}

#[deny(unused_variables)]
fn generate_main_batch(
    store: &dyn SubstateDatabase,
    DugongSettings {
        native_entity_metadata_updates,
        vm_boot_wasm_new_features,
    }: &DugongSettings,
) -> ProtocolUpdateBatch {
    let mut batch = ProtocolUpdateBatch::empty();
    let _ = store;

    if let UpdateSetting::Enabled(NoSettings) = &native_entity_metadata_updates {
        batch.mut_add_flash(
            "dugong-native-entity-metadata-updates",
            generate_dugong_native_metadata_updates(),
        );
    }

    if let UpdateSetting::Enabled(NoSettings) = &vm_boot_wasm_new_features {
        batch.mut_add_flash(
            "dugong-vm-boot-wasm-new-features",
            generate_dugong_vm_boot_wasm_new_features(),
        );
    }

    batch
}

fn generate_dugong_native_metadata_updates() -> StateUpdates {
    let mut metadata_updates: IndexMap<NodeId, MetadataInit> = Default::default();
    metadata_updates.insert(
        METADATA_MODULE_PACKAGE.into_node_id(),
        metadata_init! {
            "name" => "Metadata Module Package", locked;
            "description" => "A native package that defines the logic of the metadata module which is attached to global objects. The metadata module allows for setting and reading metadata.", locked;
        },
    );
    metadata_updates.insert(
        ROLE_ASSIGNMENT_MODULE_PACKAGE.into_node_id(),
        metadata_init! {
            "name" => "Role Assignment Module Package", locked;
            "description" => "A native package that defines the logic of the role assignments module which is attached to global objects. The role assignments module is used by the system to set and resolve the access roles for entity roles.", locked;
        },
    );
    metadata_updates.insert(
        ROYALTY_MODULE_PACKAGE.into_node_id(),
        metadata_init! {
            "name" => "Royalty Module Package", locked;
            "description" => "A native package that defines the logic of the royalty module which is optionally attached to global components if they enable royalties. The royalties module is used to configure and claim component royalties.", locked;
        },
    );

    let mut state_updates = StateUpdates::empty();
    for (node_id, metadata_updates) in metadata_updates.into_iter() {
        let node_updates = create_metadata_substates(metadata_updates).into_node_state_updates();
        state_updates.mut_add_node_updates(node_id, node_updates);
    }

    state_updates
}

fn generate_dugong_vm_boot_wasm_new_features() -> StateUpdates {
    StateUpdates::empty().set_substate(
        TRANSACTION_TRACKER,
        BOOT_LOADER_PARTITION,
        BootLoaderField::VmBoot,
        VmBoot::V1 {
            scrypto_version: ScryptoVmVersion::wasm_new_features().into(),
        },
    )
}
