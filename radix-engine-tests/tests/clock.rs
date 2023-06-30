use radix_engine::types::*;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::blueprints::consensus_manager::{
    ConsensusManagerNextRoundInput, CONSENSUS_MANAGER_NEXT_ROUND_IDENT,
};
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn sdk_clock_reads_timestamp_set_by_validator_next_round() {
    // Arrange
    let mut test_runner = TestRunner::builder()
        .with_custom_genesis(CustomGenesis::default(
            Epoch::of(1),
            CustomGenesis::default_consensus_manager_config(),
        ))
        .build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/clock");

    let time_to_set_ms = 1669663688996;
    let expected_unix_time_rounded_to_minutes = 1669663680;

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_method(
            CONSENSUS_MANAGER,
            CONSENSUS_MANAGER_NEXT_ROUND_IDENT,
            to_manifest_value_and_unwrap!(&ConsensusManagerNextRoundInput::successful(
                Round::of(1),
                0,
                time_to_set_ms,
            )),
        )
        .call_function(
            package_address,
            "ClockTest",
            "get_current_time_rounded_to_minutes",
            manifest_args![],
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![AuthAddresses::validator_role()]);

    // Assert
    let current_unix_time_rounded_to_minutes: i64 = receipt.expect_commit(true).output(2);
    assert_eq!(
        current_unix_time_rounded_to_minutes,
        expected_unix_time_rounded_to_minutes
    );
}

#[test]
fn no_auth_required_to_get_current_time_rounded_to_minutes() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/clock");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(
            package_address,
            "ClockTest",
            "get_current_time_rounded_to_minutes",
            manifest_args![],
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let current_time_rounded_to_minutes: i64 = receipt.expect_commit(true).output(1);
    assert_eq!(current_time_rounded_to_minutes, 0);
}

#[test]
fn sdk_clock_compares_against_timestamp_set_by_validator_next_round() {
    // Arrange
    let mut test_runner = TestRunner::builder()
        .with_custom_genesis(CustomGenesis::default(
            Epoch::of(1),
            CustomGenesis::default_consensus_manager_config(),
        ))
        .build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/clock");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_method(
            CONSENSUS_MANAGER,
            CONSENSUS_MANAGER_NEXT_ROUND_IDENT,
            to_manifest_value_and_unwrap!(&ConsensusManagerNextRoundInput::successful(
                Round::of(1),
                0,
                1669663688996,
            )),
        )
        .call_function(
            package_address,
            "ClockTest",
            "test_clock_comparison_operators",
            manifest_args![],
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![AuthAddresses::validator_role()]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_date_time_conversions() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/clock");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(
            package_address,
            "ClockTest",
            "test_date_time_conversions",
            manifest_args![],
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}
