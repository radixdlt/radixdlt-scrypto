use radix_engine::engine::{ModuleError, RuntimeError};
use radix_engine::types::*;
use radix_engine_interface::modules::auth::AuthAddresses;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::{Instruction, SystemTransaction};

#[test]
fn a_new_clock_instance_can_be_created_by_the_system() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let mut pre_allocated_ids = BTreeSet::new();
    pre_allocated_ids.insert(RENodeId::Global(GlobalAddress::Component(CLOCK)));
    let instructions = vec![Instruction::System(NativeInvocation::Clock(
        ClockInvocation::Create(ClockCreateInvocation {
            component_address: CLOCK.raw(),
        }),
    ))];
    let blobs = vec![];
    let receipt = test_runner.execute_transaction(
        SystemTransaction {
            instructions,
            blobs,
            nonce: 0,
            pre_allocated_ids,
        }
        .get_executable(vec![AuthAddresses::system_role()]),
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn a_new_clock_instance_cannot_be_created_by_a_validator() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let mut pre_allocated_ids = BTreeSet::new();
    pre_allocated_ids.insert(RENodeId::Global(GlobalAddress::Component(CLOCK)));
    let instructions = vec![Instruction::System(NativeInvocation::Clock(
        ClockInvocation::Create(ClockCreateInvocation {
            component_address: CLOCK.raw(),
        }),
    ))];
    let blobs = vec![];
    let receipt = test_runner.execute_transaction(
        SystemTransaction {
            instructions,
            blobs,
            nonce: 0,
            pre_allocated_ids,
        }
        .get_executable(vec![]),
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError { .. }))
    });
}

#[test]
fn set_current_time_should_fail_without_validator_auth() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/clock");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "ClockTest",
            "set_current_time",
            args!(CLOCK, 123 as i64),
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
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/clock");

    let time_to_set_ms: i64 = 1669663688996;
    let expected_unix_time_rounded_to_minutes: i64 = 1669663680;

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "ClockTest",
            "set_current_time",
            args!(CLOCK, time_to_set_ms),
        )
        .call_function(
            package_address,
            "ClockTest",
            "get_current_time_rounded_to_minutes",
            args![],
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![AuthAddresses::validator_role()]);

    // Assert
    receipt.expect_commit_success();
    let current_unix_time_rounded_to_minutes: i64 = receipt.output(2);
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
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "ClockTest",
            "get_current_time_rounded_to_minutes",
            args![],
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    let current_time_rounded_to_minutes: i64 = receipt.output(1);
    assert_eq!(current_time_rounded_to_minutes, 0);
}

#[test]
fn test_clock_comparison_methods_against_the_current_time() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/clock");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "ClockTest",
            "set_current_time",
            args!(CLOCK, 1669663688996 as i64),
        )
        .call_function(
            package_address,
            "ClockTest",
            "test_clock_comparison_operators",
            args![],
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
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "ClockTest",
            "test_date_time_conversions",
            args![],
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}
