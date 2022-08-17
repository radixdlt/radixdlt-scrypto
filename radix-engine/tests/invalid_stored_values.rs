use radix_engine::engine::RuntimeError;
use radix_engine::ledger::TypedInMemorySubstateStore;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn stored_bucket_in_committed_component_should_fail() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("stored_values");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(
            package_address,
            "InvalidInitStoredBucket",
            "create",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_failure(|e| matches!(e, RuntimeError::ValueNotAllowed));
}

#[test]
fn stored_bucket_in_owned_component_should_fail() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("stored_values");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(
            package_address,
            "InvalidStoredBucketInOwnedComponent",
            "create_bucket_in_owned_component",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_failure(|e| matches!(e, RuntimeError::ValueNotAllowed));
}
