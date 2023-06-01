use radix_engine_common::types::{Epoch, Round};
use radix_engine_interface::blueprints::consensus_manager::TimePrecision;
use radix_engine_interface::time::UtcDateTime;
use scrypto_unit::*;

#[test]
fn advancing_round_changes_app_facing_minute_resolution_clock() {
    // Arrange
    let mut test_runner = TestRunner::builder()
        .with_custom_genesis(CustomGenesis::default(
            Epoch::of(1),
            CustomGenesis::default_consensus_manager_config(),
        ))
        .build();

    let epoch_seconds_rounded_to_minutes = UtcDateTime::new(2022, 1, 1, 0, 0, 0)
        .unwrap()
        .to_instant()
        .seconds_since_unix_epoch;

    // the 13 seconds and 337 millis are supposed to be lost via rounding down to a minute
    let epoch_milli = (epoch_seconds_rounded_to_minutes + 13) * 1000 + 337;

    // Act
    test_runner
        .advance_to_round_at_timestamp(Round::of(1), epoch_milli)
        .expect_commit_success();

    // Assert
    assert_eq!(
        test_runner
            .get_current_time(TimePrecision::Minute)
            .seconds_since_unix_epoch,
        epoch_seconds_rounded_to_minutes
    );
}

#[test]
fn advancing_round_changes_internal_milli_timestamp() {
    // Arrange
    let mut test_runner = TestRunner::builder()
        .with_custom_genesis(CustomGenesis::default(
            Epoch::of(1),
            CustomGenesis::default_consensus_manager_config(),
        ))
        .build();
    let epoch_milli = 123456789;

    // Act
    test_runner.advance_to_round_at_timestamp(Round::of(1), epoch_milli);

    // Assert
    assert_eq!(test_runner.get_current_proposer_timestamp_ms(), epoch_milli);
}
