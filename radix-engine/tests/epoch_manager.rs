use radix_engine::engine::{ModuleError, RuntimeError};
use radix_engine::ledger::create_genesis;
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
    assert_eq!(epoch, 1);
}

#[test]
fn next_round_without_supervisor_auth_fails() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/epoch_manager");

    // Act
    let round = 9876u64;
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "EpochManagerTest",
            "next_round",
            args!(EPOCH_MANAGER, round),
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
fn next_round_with_validator_auth_succeeds() {
    // Arrange
    let rounds_per_epoch = 5u64;
    let genesis = create_genesis(HashSet::new(), 1u64, rounds_per_epoch);
    let mut test_runner = TestRunner::new_with_genesis(true, genesis);

    // Act
    let instructions = vec![Instruction::System(NativeInvocation::EpochManager(
        EpochManagerInvocation::NextRound(EpochManagerNextRoundInvocation {
            receiver: EPOCH_MANAGER,
            round: rounds_per_epoch - 1,
        }),
    ))];
    let receipt = test_runner.execute_transaction(
        SystemTransaction {
            instructions,
            blobs: vec![],
            nonce: 0,
        }
        .get_executable(vec![AuthAddresses::validator_role()]),
    );

    // Assert
    receipt.expect_commit_success();
    let result = receipt.expect_commit();
    assert!(result.next_epoch.is_none());
}

#[test]
fn next_epoch_with_validator_auth_succeeds() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let genesis = create_genesis(HashSet::new(), initial_epoch, rounds_per_epoch);
    let mut test_runner = TestRunner::new_with_genesis(true, genesis);

    // Act
    let instructions = vec![Instruction::System(NativeInvocation::EpochManager(
        EpochManagerInvocation::NextRound(EpochManagerNextRoundInvocation {
            receiver: EPOCH_MANAGER,
            round: rounds_per_epoch,
        }),
    ))];
    let receipt = test_runner.execute_transaction(
        SystemTransaction {
            instructions,
            blobs: vec![],
            nonce: 0,
        }
        .get_executable(vec![AuthAddresses::validator_role()]),
    );

    // Assert
    receipt.expect_commit_success();
    let result = receipt.expect_commit();
    let next_epoch = result
        .next_epoch
        .as_ref()
        .expect("Should have next epoch")
        .1;
    assert_eq!(next_epoch, initial_epoch + 1);
}

#[test]
fn epoch_manager_create_should_fail_with_supervisor_privilege() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    let instructions = vec![Instruction::System(NativeInvocation::EpochManager(
        EpochManagerInvocation::Create(EpochManagerCreateInvocation {
            validator_set: HashSet::new(),
            initial_epoch: 1u64,
            rounds_per_epoch: 1u64,
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
            validator_set: HashSet::new(),
            initial_epoch: 1u64,
            rounds_per_epoch: 1u64,
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
