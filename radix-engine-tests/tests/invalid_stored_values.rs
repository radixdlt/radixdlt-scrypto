use radix_engine::errors::{RuntimeError, SystemError};
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn stored_bucket_in_committed_component_should_fail() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/stored_values");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "InvalidInitStoredBucket",
            "create",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::PersistenceProhibited)
        )
    });
}

#[test]
fn stored_bucket_in_owned_component_should_fail() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/stored_values");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "InvalidStoredBucketInOwnedComponent",
            "create_bucket_in_owned_component",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::PersistenceProhibited)
        )
    });
}
