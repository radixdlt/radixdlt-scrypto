use radix_engine::engine::{ExecutionMode, KernelError, REActor, ResolvedFunction, RuntimeError};
use radix_engine::types::*;
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::data::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn dangling_component_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/leaks");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package_address, "Leaks", "dangling_component", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::InvalidDropNodeVisibility {
                mode: ExecutionMode::Application,
                actor: REActor::Function(ResolvedFunction::Scrypto { .. }),
                node_id: RENodeId::Component(..)
            })
        )
    });
}

#[test]
fn dangling_bucket_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/leaks");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package_address, "Leaks", "dangling_bucket", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::InvalidDropNodeVisibility {
                mode: ExecutionMode::Application,
                actor: REActor::Function(ResolvedFunction::Scrypto { .. }),
                node_id: RENodeId::Bucket(..)
            })
        )
    });
}

#[test]
fn dangling_vault_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/leaks");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package_address, "Leaks", "dangling_vault", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::InvalidDropNodeVisibility {
                mode: ExecutionMode::Application,
                actor: REActor::Function(ResolvedFunction::Scrypto { .. }),
                node_id: RENodeId::Vault(..)
            })
        )
    });
}

#[test]
fn dangling_worktop_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/leaks");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package_address, "Leaks", "get_bucket", args!())
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
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/leaks");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package_address, "Leaks", "dangling_kv_store", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::InvalidDropNodeVisibility {
                mode: ExecutionMode::Application,
                actor: REActor::Function(ResolvedFunction::Scrypto { .. }),
                node_id: RENodeId::KeyValueStore(..)
            })
        )
    });
}

#[test]
fn dangling_bucket_with_proof_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/leaks");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "Leaks",
            "dangling_bucket_with_proof",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::InvalidDropNodeVisibility {
                mode: ExecutionMode::Application,
                actor: REActor::Function(ResolvedFunction::Scrypto { .. }),
                node_id: RENodeId::Bucket(..)
            })
        )
    });
}
