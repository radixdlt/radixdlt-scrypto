use radix_engine::engine::{ModuleError, RuntimeError};
use radix_engine::types::*;
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::data::*;
use radix_engine_interface::modules::auth::AuthAddresses;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::{Instruction, SystemTransaction};

#[test]
fn get_epoch_should_succeed() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/epoch_manager");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package_address, "EpochManagerTest", "get_epoch", args![])
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let epoch: u64 = receipt.output(1);
    assert_eq!(epoch, 0);
}

#[test]
fn set_epoch_without_supervisor_auth_fails() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/epoch_manager");

    // Act
    let epoch = 9876u64;
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "EpochManagerTest",
            "set_epoch",
            args!(EPOCH_MANAGER, epoch),
        )
        .call_function(package_address, "EpochManagerTest", "get_epoch", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError { .. }))
    });
}

#[test]
fn epoch_manager_create_should_fail_with_supervisor_privilege() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    let instructions = vec![Instruction::System(NativeInvocation::EpochManager(
        EpochManagerInvocation::Create(EpochManagerCreateInvocation {
            validator_set: Vec::new(),
        }),
    ))];
    let blobs = vec![];
    let receipt = test_runner.execute_transaction(
        SystemTransaction {
            instructions,
            blobs,
            nonce: 0,
        }
        .get_executable(vec![]),
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError { .. }))
    });
}

#[test]
fn epoch_manager_create_should_succeed_with_system_privilege() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    let instructions = vec![Instruction::System(NativeInvocation::EpochManager(
        EpochManagerInvocation::Create(EpochManagerCreateInvocation {
            validator_set: Vec::new(),
        }),
    ))];
    let blobs = vec![];
    let receipt = test_runner.execute_transaction(
        SystemTransaction {
            instructions,
            blobs,
            nonce: 0,
        }
        .get_executable(vec![AuthAddresses::system_role()]),
    );

    // Assert
    receipt.expect_commit_success();
}
