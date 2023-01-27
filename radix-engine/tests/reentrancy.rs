use radix_engine::engine::{KernelError, LockState, RuntimeError, TrackError};
use radix_engine::types::*;
use radix_engine_interface::api::types::RENodeId;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn mut_reentrancy_should_not_be_possible() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/reentrancy");
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10u32.into())
        .call_function(package_address, "ReentrantComponent", "new", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component_address = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10u32.into())
        .call_method(component_address, "call_mut_self", args!(component_address))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::TrackError(TrackError::SubstateLocked(
                SubstateId(
                    RENodeId::Component(..),
                    SubstateOffset::Component(ComponentOffset::State)
                ),
                LockState::Write
            )))
        )
    });
}

#[test]
fn read_reentrancy_should_be_possible() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/reentrancy");
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10u32.into())
        .call_function(package_address, "ReentrantComponent", "new", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component_address = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10u32.into())
        .call_method(component_address, "call_self", args!(component_address))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn read_then_mut_reentrancy_should_not_be_possible() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/reentrancy");
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10u32.into())
        .call_function(package_address, "ReentrantComponent", "new", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component_address = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10u32.into())
        .call_method(
            component_address,
            "call_mut_self_2",
            args!(component_address),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::TrackError(TrackError::SubstateLocked(
                SubstateId(
                    RENodeId::Component(..),
                    SubstateOffset::Component(ComponentOffset::State)
                ),
                LockState::Read(1),
            )))
        )
    });
}
