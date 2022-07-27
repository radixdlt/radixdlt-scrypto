#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::engine::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use scrypto::core::Network;
use scrypto::prelude::*;
use scrypto::to_struct;
use transaction::builder::ManifestBuilder;

#[test]
fn test_get_epoch() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("system");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_function(package_address, "SystemTest", "get_epoch", to_struct![])
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let outputs = receipt.expect_success();
    let epoch: u64 = scrypto_decode(&outputs[0]).unwrap();
    assert_eq!(epoch, 0);
}

#[test]
fn test_set_epoch_without_system_auth_fails() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("system");

    // Act
    let epoch = 9876u64;
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_function(
            package_address,
            "SystemTest",
            "set_epoch",
            to_struct!(epoch),
        )
        .call_function(package_address, "SystemTest", "get_epoch", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_err(|e| matches!(e, RuntimeError::AuthorizationError { .. }));
}
