use scrypto_test::prelude::*;

#[test]
fn consensus_manager_min_rounds_per_epoch_is_1_in_bottlenose_ledger_simulator() {
    // Arrange
    let ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| builder.from_bootstrap_to(ProtocolVersion::Bottlenose))
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
    assert_eq!(
        dbg!(config).config.epoch_change_condition.min_round_count,
        1
    );
}

#[test]
fn consensus_manager_min_rounds_per_epoch_is_100_in_cuttlefish() {
    // Arrange
    let ledger = LedgerSimulatorBuilder::new().build();
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
