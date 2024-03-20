use radix_common::prelude::*;
use radix_engine::errors::{ApplicationError, RuntimeError};
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

#[test]
fn should_error_if_trying_to_cast_to_invalid_type() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("cast"));

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
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::PanicMessage(..))
        )
    });
}

#[test]
fn should_succeed_if_trying_to_cast_to_any() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("cast"));

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
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}
