use radix_engine::engine::{ModuleError, RuntimeError};
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::data::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::AuthModule;

#[test]
fn a_new_clock_instance_can_be_created_by_the_system() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_native_function(CLOCK_BLUEPRINT, ClockFunction::Create.as_ref(), args!())
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![AuthModule::system_role_non_fungible_address()],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn a_new_clock_instance_cannot_be_created_by_a_validator() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_native_function(CLOCK_BLUEPRINT, ClockFunction::Create.as_ref(), args!())
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![AuthModule::validator_role_non_fungible_address()],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError { .. }))
    });
}

#[test]
fn set_current_time_should_fail_without_validator_auth() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/clock");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "ClockTest",
            "set_current_time",
            args!(CLOCK, 123 as u64),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError { .. }))
    });
}

#[test]
fn validator_can_set_current_time() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/clock");

    let time_to_set_in_millis: u64 = 1669663688996;
    let expected_time_in_minutes: u64 = 27827728;

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "ClockTest",
            "set_current_time",
            args!(CLOCK, time_to_set_in_millis),
        )
        .call_function(
            package_address,
            "ClockTest",
            "get_current_time_in_minutes",
            args![],
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![AuthModule::validator_role_non_fungible_address()],
    );

    // Assert
    let outputs = receipt.expect_commit_success();
    let current_time_in_minutes: u64 = scrypto_decode(&outputs[2]).unwrap();
    assert_eq!(current_time_in_minutes, expected_time_in_minutes);
}

#[test]
fn no_auth_required_to_get_current_time_in_minutes() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/clock");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "ClockTest",
            "get_current_time_in_minutes",
            args![],
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let outputs = receipt.expect_commit_success();
    let current_time_in_minutes: u64 = scrypto_decode(&outputs[1]).unwrap();
    assert_eq!(current_time_in_minutes, 0);
}
