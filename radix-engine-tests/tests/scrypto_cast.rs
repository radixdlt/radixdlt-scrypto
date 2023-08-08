use radix_engine::errors::{ApplicationError, RuntimeError};
use radix_engine::types::*;
use scrypto::prelude::ComponentCastError;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn should_error_if_trying_to_cast_to_invalid_type() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/cast");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "CastTest",
            "cast_to_validator",
            manifest_args!(FAUCET),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::Panic(..))
        )
    });
}

#[test]
fn should_succeed_if_trying_to_cast_to_any() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/cast");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "CastTest",
            "cast_to_any",
            manifest_args!(FAUCET),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}
