use radix_engine::errors::{KernelError, RuntimeError};
use radix_engine::kernel::actor::{ExecutionMode, ResolvedActor};
use radix_engine::types::*;
use radix_engine_interface::api::types::RENodeId;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::data::manifest_args;

#[test]
fn dangling_component_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/leaks");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "Leaks",
            "dangling_component",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::InvalidDropNodeAccess {
                mode: ExecutionMode::AutoDrop,
                actor: ResolvedActor { receiver: None, .. },
                node_id: RENodeId::Component(..)
            })
        )
    });
}

#[test]
fn dangling_bucket_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/leaks");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "Leaks",
            "dangling_bucket",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::InvalidDropNodeAccess {
                mode: ExecutionMode::AutoDrop,
                actor: ResolvedActor { receiver: None, .. },
                node_id: RENodeId::Bucket(..)
            })
        )
    });
}

#[test]
fn dangling_vault_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/leaks");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package_address, "Leaks", "dangling_vault", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::InvalidDropNodeAccess {
                mode: ExecutionMode::AutoDrop,
                actor: ResolvedActor { receiver: None, .. },
                node_id: RENodeId::Vault(..)
            })
        )
    });
}

#[test]
fn dangling_worktop_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/leaks");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package_address, "Leaks", "get_bucket", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::DropNodeFailure(RENodeId::Worktop))
        )
    });
}

#[test]
fn dangling_kv_store_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/leaks");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "Leaks",
            "dangling_kv_store",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::InvalidDropNodeAccess {
                mode: ExecutionMode::AutoDrop,
                actor: ResolvedActor { receiver: None, .. },
                node_id: RENodeId::KeyValueStore(..)
            })
        )
    });
}

#[test]
fn dangling_bucket_with_proof_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/leaks");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "Leaks",
            "dangling_bucket_with_proof",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::InvalidDropNodeAccess {
                mode: ExecutionMode::AutoDrop,
                actor: ResolvedActor { receiver: None, .. },
                node_id: RENodeId::Bucket(..)
            })
        )
    });
}
