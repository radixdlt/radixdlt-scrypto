use radix_engine::engine::{ModuleError, RuntimeError};
use radix_engine::ledger::TypedInMemorySubstateStore;
use scrypto::core::Network;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_get_epoch() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("system");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(package_address, "SystemTest", "get_epoch", args![])
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let outputs = receipt.expect_success();
    let epoch: u64 = scrypto_decode(&outputs[1]).unwrap();
    assert_eq!(epoch, 0);
}

#[test]
fn test_set_epoch_without_system_auth_fails() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("system");

    // Act
    let epoch = 9876u64;
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(package_address, "SystemTest", "set_epoch", args!(epoch))
        .call_function(package_address, "SystemTest", "get_epoch", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_failure(|e| {
        matches!(
            e,
            RuntimeError::ModuleError(ModuleError::AuthorizationError { .. })
        )
    });
}
