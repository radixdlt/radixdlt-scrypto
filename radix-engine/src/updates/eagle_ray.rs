use super::*;
use crate::{internal_prelude::*, system::system_callback::SystemBoot};

#[derive(Clone, ScryptoSbor)]
pub struct EagleRaySettings {
    pub system_version_update: UpdateSetting<NoSettings>,
}

impl UpdateSettings for EagleRaySettings {
    type UpdateGenerator = EagleRayGenerator;

    fn protocol_version() -> ProtocolVersion {
        ProtocolVersion::EagleRay
    }

    fn all_enabled_as_default_for_network(network: &NetworkDefinition) -> Self {
        Self {
            system_version_update: UpdateSetting::enabled_as_default_for_network(network),
        }
    }

    fn all_disabled() -> Self {
        Self {
            system_version_update: UpdateSetting::Disabled,
        }
    }

    fn create_generator(&self) -> Self::UpdateGenerator {
        Self::UpdateGenerator {
            settings: self.clone(),
        }
    }
}

pub struct EagleRayGenerator {
    settings: EagleRaySettings,
}

impl ProtocolUpdateGenerator for EagleRayGenerator {
    fn batch_groups(&self) -> Vec<Box<dyn ProtocolUpdateBatchGroupGenerator<'_> + '_>> {
        vec![FixedBatchGroupGenerator::named("principal")
            .add_batch("primary", |store| {
                generate_main_batch(store, &self.settings)
            })
            .build()]
    }
}

fn generate_main_batch(
    store: &dyn SubstateDatabase,
    EagleRaySettings {
        system_version_update,
    }: &EagleRaySettings,
) -> ProtocolUpdateBatch {
    let mut batch = ProtocolUpdateBatch::empty();

    if let UpdateSetting::Enabled(NoSettings) = system_version_update {
        batch.mut_add_flash(
            "eagle-ray-system-version-update",
            generate_system_boot_v5_updates(store),
        );
    }

    batch
}

fn generate_system_boot_v5_updates(store: &dyn SubstateDatabase) -> StateUpdates {
    let existing_system_boot: SystemBoot = store.get_existing_substate(
        TRANSACTION_TRACKER,
        BOOT_LOADER_PARTITION,
        BootLoaderField::SystemBoot,
    );

    StateUpdates::empty().set_substate(
        TRANSACTION_TRACKER,
        BOOT_LOADER_PARTITION,
        BootLoaderField::SystemBoot,
        SystemBoot::eagle_ray_for_previous_parameters(existing_system_boot.into_parameters()),
    )
}
