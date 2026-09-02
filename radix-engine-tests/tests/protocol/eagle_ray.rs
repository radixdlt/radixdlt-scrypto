use radix_engine_tests::prelude::*;

#[test]
fn default_eagle_ray_settings_advance_system_boot_to_v5() {
    // Arrange
    let ledger = LedgerSimulatorBuilder::new().build();

    // Act
    let system_boot = ledger.substate_db().get_existing_substate::<SystemBoot>(
        TRANSACTION_TRACKER,
        BOOT_LOADER_PARTITION,
        BootLoaderField::SystemBoot,
    );

    // Assert
    assert_eq!(
        system_boot,
        SystemBoot::eagle_ray_for_previous_parameters(SystemParameters::latest(
            NetworkDefinition::simulator(),
        )),
    );
}

#[test]
fn disabled_eagle_ray_system_update_leaves_system_boot_at_v3() {
    // Arrange
    let ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| {
            builder
                .configure_eagle_ray(|settings| {
                    settings.disable(|settings| &mut settings.system_version_update)
                })
                .from_bootstrap_to(ProtocolVersion::EagleRay)
        })
        .build();

    // Act
    let system_boot = ledger.substate_db().get_existing_substate::<SystemBoot>(
        TRANSACTION_TRACKER,
        BOOT_LOADER_PARTITION,
        BootLoaderField::SystemBoot,
    );

    // Assert
    assert_eq!(
        system_boot,
        SystemBoot::cuttlefish(NetworkDefinition::simulator()),
    );
}
