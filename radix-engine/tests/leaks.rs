use radix_engine::engine::{DropFailure, KernelError, RuntimeError};
use radix_engine::ledger::TypedInMemorySubstateStore;
use scrypto::core::Network;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn dangling_component_should_fail() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("leaks");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(package_address, "Leaks", "dangling_component", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::DropFailure(DropFailure::Component))
        )
    });
}

#[test]
fn dangling_bucket_should_fail() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("leaks");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(package_address, "Leaks", "dangling_bucket", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::DropFailure(DropFailure::Bucket))
        )
    });
}

#[test]
fn dangling_vault_should_fail() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("leaks");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(package_address, "Leaks", "dangling_vault", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::DropFailure(DropFailure::Vault))
        )
    });
}

#[test]
fn dangling_worktop_should_fail() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("leaks");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(package_address, "Leaks", "get_bucket", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::DropFailure(DropFailure::Worktop))
        )
    });
}

#[test]
fn dangling_kv_store_should_fail() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("leaks");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(package_address, "Leaks", "dangling_kv_store", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::DropFailure(DropFailure::KeyValueStore))
        )
    });
}

#[test]
fn dangling_bucket_with_proof_should_fail() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("leaks");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(
            package_address,
            "Leaks",
            "dangling_bucket_with_proof",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::DropFailure(DropFailure::Bucket))
        )
    });
}
