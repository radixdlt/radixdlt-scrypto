use scrypto_test::prelude::*;

#[test]
fn consensus_manager_min_rounds_per_epoch_is_100_in_cuttlefish() {
    // Arrange
    let ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| {
            builder
                .configure_cuttlefish(|mut creator| {
                    creator.update_number_of_min_rounds_per_epoch =
                        UpdateSetting::Enabled(UpdateNumberOfMinRoundsPerEpochSettings::Set {
                            value: 100,
                        });
                    creator
                })
                .from_bootstrap_to(ProtocolVersion::Cuttlefish)
        })
        .build();
    let database = ledger.substate_db();

    // Act
    let config = database
        .get_existing_substate::<FieldSubstate<VersionedConsensusManagerConfiguration>>(
            CONSENSUS_MANAGER,
            MAIN_BASE_PARTITION,
            ConsensusManagerField::Configuration,
        )
        .into_payload()
        .fully_update_and_into_latest_version();

    // Assert
    assert_eq!(config.config.epoch_change_condition.min_round_count, 100);
}
