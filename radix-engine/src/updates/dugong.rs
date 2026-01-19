use super::*;
use crate::{internal_prelude::*, system::system_callback::SystemBoot};

#[derive(Clone, ScryptoSbor)]
pub struct DugongSettings {
    pub native_entity_metadata_updates: UpdateSetting<NoSettings>,
    pub system_logic_updates: UpdateSetting<NoSettings>,
}

impl UpdateSettings for DugongSettings {
    type UpdateGenerator = DugongGenerator;

    fn protocol_version() -> ProtocolVersion {
        ProtocolVersion::Dugong
    }

    /// # Note
    ///
    /// For the time being this function creates a new [`DugongSettings`] with all of the updates
    /// disabled. This is done because this version of Scrypto ships with the Dugong protocol update
    /// but we won't enact the protocol upgrade.
    fn all_enabled_as_default_for_network(_network: &NetworkDefinition) -> Self {
        Self::all_disabled()
    }

    fn all_disabled() -> Self {
        Self {
            native_entity_metadata_updates: UpdateSetting::Disabled,
            system_logic_updates: UpdateSetting::Disabled,
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
    fn batch_groups(&self) -> Vec<Box<dyn ProtocolUpdateBatchGroupGenerator<'_> + '_>> {
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
        system_logic_updates,
    }: &DugongSettings,
) -> ProtocolUpdateBatch {
    let mut batch = ProtocolUpdateBatch::empty();

    if let UpdateSetting::Enabled(NoSettings) = &native_entity_metadata_updates {
        batch.mut_add_flash(
            "dugong-native-entity-metadata-updates",
            generate_dugong_native_metadata_updates(),
        );
    }

    if let UpdateSetting::Enabled(NoSettings) = &system_logic_updates {
        batch.mut_add_flash(
            "dugong-system-logic-updates",
            generate_system_logic_v4_updates(store),
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

fn generate_system_logic_v4_updates(store: &dyn SubstateDatabase) -> StateUpdates {
    let existing_system_boot: SystemBoot = store.get_existing_substate(
        TRANSACTION_TRACKER,
        BOOT_LOADER_PARTITION,
        BootLoaderField::SystemBoot,
    );

    StateUpdates::empty().set_substate(
        TRANSACTION_TRACKER,
        BOOT_LOADER_PARTITION,
        BootLoaderField::SystemBoot,
        SystemBoot::dugong_for_previous_parameters(existing_system_boot.into_parameters()),
    )
}
