#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::engine::{DropFailure, RuntimeError};
use scrypto::core::Network;
use scrypto::prelude::*;
use scrypto::to_struct;
use transaction::builder::ManifestBuilder;

#[test]
fn dangling_component_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("leaks");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_function(package_address, "Leaks", "dangling_component", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_err(|e| matches!(e, RuntimeError::DropFailure(DropFailure::Component)));
}

#[test]
fn dangling_bucket_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("leaks");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_function(package_address, "Leaks", "dangling_bucket", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_err(|e| matches!(e, RuntimeError::DropFailure(DropFailure::Bucket)));
}

#[test]
fn dangling_vault_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("leaks");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_function(package_address, "Leaks", "dangling_vault", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_err(|e| matches!(e, RuntimeError::DropFailure(DropFailure::Vault)));
}

#[test]
fn dangling_worktop_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("leaks");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_function(package_address, "Leaks", "get_bucket", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_err(|e| matches!(e, RuntimeError::DropFailure(DropFailure::Worktop)));
}

#[test]
fn dangling_kv_store_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("leaks");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_function(package_address, "Leaks", "dangling_kv_store", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_err(|e| matches!(e, RuntimeError::DropFailure(DropFailure::KeyValueStore)));
}

#[test]
fn dangling_bucket_with_proof_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("leaks");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_function(
            package_address,
            "Leaks",
            "dangling_bucket_with_proof",
            to_struct!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_err(|e| matches!(e, RuntimeError::DropFailure(DropFailure::Bucket)));
}
