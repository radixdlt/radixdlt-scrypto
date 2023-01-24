use radix_engine::blueprints::epoch_manager::Validator;
use radix_engine::errors::{ModuleError, RuntimeError};
use radix_engine::ledger::create_genesis;
use radix_engine::types::*;
use radix_engine_interface::modules::auth::AuthAddresses;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::{Instruction, SystemTransaction};
use transaction::signing::EcdsaSecp256k1PrivateKey;

#[test]
fn get_epoch_should_succeed() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/epoch_manager");

    // Act
    let manifest = ManifestBuilder::new()
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
    let manifest = ManifestBuilder::new()
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
    let genesis = create_genesis(BTreeMap::new(), 1u64, rounds_per_epoch);
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
            pre_allocated_ids: BTreeSet::new(),
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
    let genesis = create_genesis(BTreeMap::new(), initial_epoch, rounds_per_epoch);
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
            pre_allocated_ids: BTreeSet::new(),
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
fn register_validator_with_auth_succeeds() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let pub_key = EcdsaSecp256k1PrivateKey::from_u64(1u64)
        .unwrap()
        .public_key();
    let mut validator_set = BTreeMap::new();
    validator_set.insert(pub_key, Decimal::one());
    let genesis = create_genesis(validator_set, initial_epoch, rounds_per_epoch);
    let mut test_runner = TestRunner::new_with_genesis(true, genesis);

    // Act
    let validator_address = test_runner.get_validator_with_key(&pub_key);
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .register_validator(validator_address)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn register_validator_without_auth_fails() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let pub_key = EcdsaSecp256k1PrivateKey::from_u64(1u64)
        .unwrap()
        .public_key();
    let mut validator_set = BTreeMap::new();
    validator_set.insert(pub_key, Decimal::one());
    let genesis = create_genesis(validator_set, initial_epoch, rounds_per_epoch);
    let mut test_runner = TestRunner::new_with_genesis(true, genesis);

    // Act
    let validator_address = test_runner.get_validator_with_key(&pub_key);
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .register_validator(validator_address)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError(..)))
    });
}

#[test]
fn unregister_validator_with_auth_succeeds() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let pub_key = EcdsaSecp256k1PrivateKey::from_u64(1u64)
        .unwrap()
        .public_key();
    let mut validator_set = BTreeMap::new();
    validator_set.insert(pub_key, Decimal::one());
    let genesis = create_genesis(validator_set, initial_epoch, rounds_per_epoch);
    let mut test_runner = TestRunner::new_with_genesis(true, genesis);

    // Act
    let validator_address = test_runner.get_validator_with_key(&pub_key);
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .unregister_validator(validator_address)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn unregister_validator_without_auth_fails() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let pub_key = EcdsaSecp256k1PrivateKey::from_u64(1u64)
        .unwrap()
        .public_key();
    let mut validator_set = BTreeMap::new();
    validator_set.insert(pub_key, Decimal::one());
    let genesis = create_genesis(validator_set, initial_epoch, rounds_per_epoch);
    let mut test_runner = TestRunner::new_with_genesis(true, genesis);

    // Act
    let validator_address = test_runner.get_validator_with_key(&pub_key);
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .unregister_validator(validator_address)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError(..)))
    });
}

#[test]
fn registered_validator_with_no_stake_does_not_become_part_of_validator_on_epoch_change() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let genesis = create_genesis(BTreeMap::new(), initial_epoch, rounds_per_epoch);
    let mut test_runner = TestRunner::new_with_genesis(true, genesis);
    let (pub_key, validator_address) = test_runner.new_validator();
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .register_validator(validator_address)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );
    receipt.expect_commit_success();

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
            pre_allocated_ids: BTreeSet::new(),
        }
        .get_executable(vec![AuthAddresses::validator_role()]),
    );

    // Assert
    receipt.expect_commit_success();
    let result = receipt.expect_commit();
    let next_epoch = result.next_epoch.as_ref().expect("Should have next epoch");
    assert_eq!(next_epoch.1, initial_epoch + 1);
    assert!(!next_epoch.0.contains_key(&validator_address));
}

#[test]
fn registered_validator_with_stake_does_become_part_of_validator_on_epoch_change() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let genesis = create_genesis(BTreeMap::new(), initial_epoch, rounds_per_epoch);
    let mut test_runner = TestRunner::new_with_genesis(true, genesis);
    let (pub_key, _, account_address) = test_runner.new_account(false);
    let validator_address = test_runner.new_validator_with_pub_key(pub_key);
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .withdraw_from_account_by_amount(account_address, Decimal::one(), RADIX_TOKEN)
        .register_validator(validator_address)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.stake_validator(validator_address, bucket_id)
        })
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );
    receipt.expect_commit_success();

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
            pre_allocated_ids: BTreeSet::new(),
        }
        .get_executable(vec![AuthAddresses::validator_role()]),
    );

    // Assert
    receipt.expect_commit_success();
    let result = receipt.expect_commit();
    let next_epoch = result.next_epoch.as_ref().expect("Should have next epoch");
    assert_eq!(next_epoch.1, initial_epoch + 1);
    assert_eq!(
        next_epoch.0.get(&validator_address).unwrap(),
        &Validator {
            key: pub_key,
            stake: Decimal::one(),
        }
    );
}

#[test]
fn unregistered_validator_gets_removed_on_epoch_change() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let pub_key = EcdsaSecp256k1PrivateKey::from_u64(1u64)
        .unwrap()
        .public_key();
    let mut validator_set = BTreeMap::new();
    validator_set.insert(pub_key, Decimal::one());
    let genesis = create_genesis(validator_set, initial_epoch, rounds_per_epoch);
    let mut test_runner = TestRunner::new_with_genesis(true, genesis);
    let validator_address = test_runner.get_validator_with_key(&pub_key);
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .unregister_validator(validator_address)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );
    receipt.expect_commit_success();

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
            pre_allocated_ids: BTreeSet::new(),
        }
        .get_executable(vec![AuthAddresses::validator_role()]),
    );

    // Assert
    receipt.expect_commit_success();
    let result = receipt.expect_commit();
    let next_epoch = result.next_epoch.as_ref().expect("Should have next epoch");
    assert_eq!(next_epoch.1, initial_epoch + 1);
    assert!(!next_epoch.0.contains_key(&validator_address));
}

#[test]
fn unstaked_validator_gets_less_stake_on_epoch_change() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let pub_key = EcdsaSecp256k1PrivateKey::from_u64(1u64)
        .unwrap()
        .public_key();
    let mut validator_set = BTreeMap::new();
    validator_set.insert(pub_key, Decimal::from(10));
    let genesis = create_genesis(validator_set, initial_epoch, rounds_per_epoch);
    let mut test_runner = TestRunner::new_with_genesis(true, genesis);
    let (_, _, account_address) = test_runner.new_account(true);
    let validator_address = test_runner.get_validator_with_key(&pub_key);
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .unstake_validator(validator_address, Decimal::one())
        .call_method(
            account_address,
            "deposit_batch",
            args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );
    receipt.expect_commit_success();

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
            pre_allocated_ids: BTreeSet::new(),
        }
        .get_executable(vec![AuthAddresses::validator_role()]),
    );

    // Assert
    receipt.expect_commit_success();
    let result = receipt.expect_commit();
    let next_epoch = result.next_epoch.as_ref().expect("Should have next epoch");
    assert_eq!(next_epoch.1, initial_epoch + 1);
    assert_eq!(
        next_epoch.0.get(&validator_address).unwrap(),
        &Validator {
            key: pub_key,
            stake: Decimal::from(9),
        }
    );
}

#[test]
fn epoch_manager_create_should_fail_with_supervisor_privilege() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    let mut pre_allocated_ids = BTreeSet::new();
    pre_allocated_ids.insert(RENodeId::Global(GlobalAddress::Component(EPOCH_MANAGER)));
    let instructions = vec![Instruction::System(NativeInvocation::EpochManager(
        EpochManagerInvocation::Create(EpochManagerCreateInvocation {
            component_address: EPOCH_MANAGER.raw(),
            validator_set: BTreeMap::new(),
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
            pre_allocated_ids,
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
    let mut pre_allocated_ids = BTreeSet::new();
    pre_allocated_ids.insert(RENodeId::Global(GlobalAddress::Component(EPOCH_MANAGER)));
    let instructions = vec![Instruction::System(NativeInvocation::EpochManager(
        EpochManagerInvocation::Create(EpochManagerCreateInvocation {
            component_address: EPOCH_MANAGER.raw(),
            validator_set: BTreeMap::new(),
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
            pre_allocated_ids,
        }
        .get_executable(vec![AuthAddresses::system_role()]),
    );

    // Assert
    receipt.expect_commit_success();
}
