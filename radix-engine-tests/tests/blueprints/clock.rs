use radix_common::prelude::*;
use radix_engine_interface::blueprints::consensus_manager::{
    ConsensusManagerNextRoundInput, CONSENSUS_MANAGER_NEXT_ROUND_IDENT,
};
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

#[test]
fn sdk_clock_reads_timestamp_set_by_validator_next_round() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("clock"));

    let time_to_set_ms = 1669663688996;
    let expected_unix_time_rounded_to_minutes = 1669663680;

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            CONSENSUS_MANAGER,
            CONSENSUS_MANAGER_NEXT_ROUND_IDENT,
            ConsensusManagerNextRoundInput::successful(Round::of(1), 0, time_to_set_ms),
        )
        .call_function(
            package_address,
            "ClockTest",
            "get_current_time_rounded_to_minutes",
            manifest_args![],
        )
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![system_execution(SystemExecution::Validator)]);

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
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("clock"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "ClockTest",
            "get_current_time_rounded_to_minutes",
            manifest_args![],
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    let current_time_rounded_to_minutes: i64 = receipt.expect_commit(true).output(1);
    assert_eq!(current_time_rounded_to_minutes, 0);
}

#[test]
fn sdk_clock_compares_against_timestamp_set_by_validator_next_round() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("clock"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            CONSENSUS_MANAGER,
            CONSENSUS_MANAGER_NEXT_ROUND_IDENT,
            ConsensusManagerNextRoundInput::successful(Round::of(1), 0, 1669663688996),
        )
        .call_function(
            package_address,
            "ClockTest",
            "test_clock_comparison_operators",
            manifest_args![],
        )
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![system_execution(SystemExecution::Validator)]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_date_time_conversions() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("clock"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "ClockTest",
            "test_date_time_conversions",
            manifest_args![],
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn compare_max_time_should_return_true() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("clock"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "ClockTest",
            "compare",
            manifest_args!(Instant::new(i64::MAX)),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    let result = receipt.expect_commit_success();
    let rtn: bool = result.output(1);
    assert!(rtn);
}

#[test]
fn compare_min_time_should_return_false() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("clock"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "ClockTest",
            "compare",
            manifest_args!(Instant::new(i64::MIN)),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    let result = receipt.expect_commit_success();
    let rtn: bool = result.output(1);
    assert!(!rtn);
}
