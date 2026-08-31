use super::*;
use crate::{internal_prelude::*, kernel::kernel::KernelBoot};

#[derive(Clone, ScryptoSbor)]
pub struct EagleRaySettings {
    pub kernel_version_update: UpdateSetting<NoSettings>,
}

impl UpdateSettings for EagleRaySettings {
    type UpdateGenerator = EagleRayGenerator;

    fn protocol_version() -> ProtocolVersion {
        ProtocolVersion::EagleRay
    }

    fn all_enabled_as_default_for_network(network: &NetworkDefinition) -> Self {
        Self {
            kernel_version_update: UpdateSetting::enabled_as_default_for_network(network),
        }
    }

    fn all_disabled() -> Self {
        Self {
            kernel_version_update: UpdateSetting::Disabled,
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
        kernel_version_update,
    }: &EagleRaySettings,
) -> ProtocolUpdateBatch {
    let mut batch = ProtocolUpdateBatch::empty();

    if let UpdateSetting::Enabled(NoSettings) = kernel_version_update {
        batch.mut_add_flash(
            "eagle-ray-kernel-version-update",
            generate_kernel_boot_v3_updates(store),
        );
    }

    batch
}

fn generate_kernel_boot_v3_updates(store: &dyn SubstateDatabase) -> StateUpdates {
    let existing_kernel_boot: KernelBoot = store.get_existing_substate(
        TRANSACTION_TRACKER,
        BOOT_LOADER_PARTITION,
        BootLoaderField::KernelBoot,
    );
    let global_nodes_version = existing_kernel_boot.always_visible_global_nodes_version();

    StateUpdates::empty().set_substate(
        TRANSACTION_TRACKER,
        BOOT_LOADER_PARTITION,
        BootLoaderField::KernelBoot,
        KernelBoot::eagle_ray_for_previous_parameters(global_nodes_version),
    )
}
