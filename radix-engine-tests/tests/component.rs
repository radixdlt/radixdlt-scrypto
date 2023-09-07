mod package_loader;

use package_loader::PackageLoader;
use radix_engine::errors::RuntimeError;
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn test_component() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let package = test_runner.publish_package_simple(PackageLoader::get("component"));

    // Create component
    let manifest1 = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "ComponentTest",
            "create_component",
            manifest_args!(),
        )
        .build();
    let receipt1 = test_runner.execute_manifest(manifest1, vec![]);
    receipt1.expect_commit_success();
}

#[test]
fn invalid_blueprint_name_should_cause_error() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_addr = test_runner.publish_package_simple(PackageLoader::get("component"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_addr,
            "NonExistentBlueprint",
            "create_component",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| matches!(e, RuntimeError::SystemError(..)));
}
