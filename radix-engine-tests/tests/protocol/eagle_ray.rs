use radix_engine::kernel::kernel::KernelBoot;
use radix_engine_tests::prelude::*;

#[test]
fn default_eagle_ray_settings_advance_kernel_boot_to_v3() {
    // Arrange
    let ledger = LedgerSimulatorBuilder::new().build();

    // Act
    let kernel_boot = KernelBoot::load(ledger.substate_db());

    // Assert
    assert_eq!(
        kernel_boot,
        KernelBoot::eagle_ray_for_previous_parameters(AlwaysVisibleGlobalNodesVersion::V2),
    );
}

#[test]
fn disabled_eagle_ray_kernel_update_leaves_kernel_boot_at_v2() {
    // Arrange
    let ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| {
            builder
                .configure_eagle_ray(|settings| {
                    settings.disable(|settings| &mut settings.kernel_version_update)
                })
                .from_bootstrap_to(ProtocolVersion::EagleRay)
        })
        .build();
    // Act
    let kernel_boot = KernelBoot::load(ledger.substate_db());
    // Assert
    assert_eq!(kernel_boot, KernelBoot::cuttlefish());
}
