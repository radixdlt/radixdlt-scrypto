#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use scrypto::prelude::*;
use scrypto::to_struct;
use transaction::builder::ManifestBuilder;
use radix_engine::engine::RuntimeError;

#[test]
fn test_get_epoch() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("system");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "SystemTest", "get_epoch", to_struct![])
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
    let epoch: u64 = scrypto_decode(&receipt.outputs[0]).unwrap();
    assert_eq!(epoch, 0);
}

#[test]
fn test_set_epoch_without_system_auth_fails() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("system");

    // Act
    let epoch = 9876u64;
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "SystemTest", "set_epoch", to_struct!(epoch))
        .call_function(package_address, "SystemTest", "get_epoch", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_err(|e| matches!(e, RuntimeError::AuthorizationError { .. }));
}
