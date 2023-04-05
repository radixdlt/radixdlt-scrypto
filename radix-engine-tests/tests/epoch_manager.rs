use radix_engine::blueprints::epoch_manager::{Validator, ValidatorError};
use radix_engine::errors::{ApplicationError, ModuleError, RuntimeError};
use radix_engine::system::bootstrap::create_genesis;
use radix_engine::system::kernel_modules::auth::AuthError;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::blueprints::epoch_manager::*;
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
use transaction::model::{Instruction, SystemTransaction};

#[test]
fn get_epoch_should_succeed() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/epoch_manager");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "EpochManagerTest",
            "get_epoch",
            manifest_args![],
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let epoch: u64 = receipt.expect_commit(true).output(1);
    assert_eq!(epoch, 1);
}

#[test]
fn next_round_without_supervisor_auth_fails() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/epoch_manager");

    // Act
    let round = 9876u64;
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "EpochManagerTest",
            "next_round",
            manifest_args!(EPOCH_MANAGER, round),
        )
        .call_function(
            package_address,
            "EpochManagerTest",
            "get_epoch",
            manifest_args!(),
        )
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
    let num_unstake_epochs = 1u64;
    let genesis = create_genesis(
        BTreeMap::new(),
        BTreeMap::new(),
        1u64,
        rounds_per_epoch,
        num_unstake_epochs,
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();

    // Act
    let instructions = vec![Instruction::CallMethod {
        component_address: EPOCH_MANAGER,
        method_name: EPOCH_MANAGER_NEXT_ROUND_IDENT.to_string(),
        args: to_manifest_value(&EpochManagerNextRoundInput {
            round: rounds_per_epoch - 1,
        }),
    }];
    let receipt = test_runner.execute_transaction(
        SystemTransaction {
            instructions,
            blobs: vec![],
            nonce: 0,
            pre_allocated_ids: BTreeSet::new(),
        }
        .get_executable(btreeset![AuthAddresses::validator_role()]),
    );

    // Assert
    let result = receipt.expect_commit(true);
    assert!(result.next_epoch().is_none());
}

#[test]
fn next_epoch_with_validator_auth_succeeds() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let num_unstake_epochs = 1u64;
    let genesis = create_genesis(
        BTreeMap::new(),
        BTreeMap::new(),
        initial_epoch,
        rounds_per_epoch,
        num_unstake_epochs,
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();

    // Act
    let instructions = vec![Instruction::CallMethod {
        component_address: EPOCH_MANAGER,
        method_name: EPOCH_MANAGER_NEXT_ROUND_IDENT.to_string(),
        args: to_manifest_value(&EpochManagerNextRoundInput {
            round: rounds_per_epoch,
        }),
    }];
    let receipt = test_runner.execute_transaction(
        SystemTransaction {
            instructions,
            blobs: vec![],
            nonce: 0,
            pre_allocated_ids: BTreeSet::new(),
        }
        .get_executable(btreeset![AuthAddresses::validator_role()]),
    );

    // Assert
    let result = receipt.expect_commit(true);
    let next_epoch = result.next_epoch().expect("Should have next epoch").1;
    assert_eq!(next_epoch, initial_epoch + 1);
}

#[test]
fn register_validator_with_auth_succeeds() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let num_unstake_epochs = 1u64;
    let pub_key = EcdsaSecp256k1PrivateKey::from_u64(1u64)
        .unwrap()
        .public_key();
    let mut validator_set_and_stake_owners = BTreeMap::new();
    let validator_account_address = ComponentAddress::virtual_account_from_public_key(&pub_key);
    validator_set_and_stake_owners.insert(pub_key, (Decimal::one(), validator_account_address));
    let genesis = create_genesis(
        validator_set_and_stake_owners,
        BTreeMap::new(),
        initial_epoch,
        rounds_per_epoch,
        num_unstake_epochs,
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();

    // Act
    let validator_address = test_runner.get_validator_with_key(&pub_key);
    let manifest = ManifestBuilder::new()
        .create_proof_from_account(validator_account_address, VALIDATOR_OWNER_TOKEN)
        .lock_fee(test_runner.faucet_component(), 10.into())
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
    let num_unstake_epochs = 1u64;
    let pub_key = EcdsaSecp256k1PrivateKey::from_u64(1u64)
        .unwrap()
        .public_key();
    let mut validator_set_and_stake_owners = BTreeMap::new();
    validator_set_and_stake_owners.insert(
        pub_key,
        (
            Decimal::one(),
            ComponentAddress::virtual_account_from_public_key(&pub_key),
        ),
    );
    let genesis = create_genesis(
        validator_set_and_stake_owners,
        BTreeMap::new(),
        initial_epoch,
        rounds_per_epoch,
        num_unstake_epochs,
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();

    // Act
    let validator_address = test_runner.get_validator_with_key(&pub_key);
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
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
    let num_unstake_epochs = 1u64;
    let pub_key = EcdsaSecp256k1PrivateKey::from_u64(1u64)
        .unwrap()
        .public_key();
    let validator_account_address = ComponentAddress::virtual_account_from_public_key(&pub_key);
    let mut validator_set_and_stake_owners = BTreeMap::new();
    validator_set_and_stake_owners.insert(pub_key, (Decimal::one(), validator_account_address));
    let genesis = create_genesis(
        validator_set_and_stake_owners,
        BTreeMap::new(),
        initial_epoch,
        rounds_per_epoch,
        num_unstake_epochs,
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();

    // Act
    let validator_address = test_runner.get_validator_with_key(&pub_key);
    let manifest = ManifestBuilder::new()
        .create_proof_from_account(validator_account_address, VALIDATOR_OWNER_TOKEN)
        .lock_fee(test_runner.faucet_component(), 10.into())
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
    let num_unstake_epochs = 1u64;
    let pub_key = EcdsaSecp256k1PrivateKey::from_u64(1u64)
        .unwrap()
        .public_key();
    let mut validator_set_and_stake_owners = BTreeMap::new();
    validator_set_and_stake_owners.insert(
        pub_key,
        (
            Decimal::one(),
            ComponentAddress::virtual_account_from_public_key(&pub_key),
        ),
    );
    let genesis = create_genesis(
        validator_set_and_stake_owners,
        BTreeMap::new(),
        initial_epoch,
        rounds_per_epoch,
        num_unstake_epochs,
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();

    // Act
    let validator_address = test_runner.get_validator_with_key(&pub_key);
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .unregister_validator(validator_address)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError(..)))
    });
}

fn test_disabled_delegated_stake(owner: bool, expect_success: bool) {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let num_unstake_epochs = 1u64;
    let pub_key = EcdsaSecp256k1PrivateKey::from_u64(1u64)
        .unwrap()
        .public_key();
    let validator_account_address = ComponentAddress::virtual_account_from_public_key(&pub_key);
    let mut validator_set_and_stake_owners = BTreeMap::new();
    validator_set_and_stake_owners.insert(pub_key, (Decimal::one(), validator_account_address));
    let genesis = create_genesis(
        validator_set_and_stake_owners,
        BTreeMap::new(),
        initial_epoch,
        rounds_per_epoch,
        num_unstake_epochs,
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let validator_address = test_runner.get_validator_with_key(&pub_key);
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account(validator_account_address, VALIDATOR_OWNER_TOKEN)
        .call_method(
            validator_address,
            "update_accept_delegated_stake",
            manifest_args!(false),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );
    receipt.expect_commit_success();

    // Act
    let mut builder = ManifestBuilder::new();
    builder.lock_fee(test_runner.faucet_component(), 10.into());

    if owner {
        builder.create_proof_from_account(validator_account_address, VALIDATOR_OWNER_TOKEN);
    }

    let manifest = builder
        .call_method(test_runner.faucet_component(), "free", manifest_args!())
        .take_from_worktop(RADIX_TOKEN, |builder, bucket| {
            builder.call_method(validator_address, "stake", manifest_args!(bucket))
        })
        .call_method(
            validator_account_address,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );

    // Assert
    if expect_success {
        receipt.expect_commit_success();
    } else {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::ModuleError(ModuleError::AuthError(AuthError::Unauthorized { .. }))
            )
        });
    }
}

#[test]
fn not_allowing_delegated_stake_should_still_let_owner_stake() {
    test_disabled_delegated_stake(true, true);
}

#[test]
fn not_allowing_delegated_stake_should_not_let_non_owner_stake() {
    test_disabled_delegated_stake(false, false);
}

#[test]
fn registered_validator_with_no_stake_does_not_become_part_of_validator_on_epoch_change() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let num_unstake_epochs = 1u64;
    let genesis = create_genesis(
        BTreeMap::new(),
        BTreeMap::new(),
        initial_epoch,
        rounds_per_epoch,
        num_unstake_epochs,
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let (pub_key, _, account_address) = test_runner.new_account(false);
    let validator_address = test_runner.new_validator_with_pub_key(pub_key, account_address);
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account(account_address, VALIDATOR_OWNER_TOKEN)
        .register_validator(validator_address)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );
    receipt.expect_commit_success();

    // Act
    let instructions = vec![Instruction::CallMethod {
        component_address: EPOCH_MANAGER,
        method_name: EPOCH_MANAGER_NEXT_ROUND_IDENT.to_string(),
        args: to_manifest_value(&EpochManagerNextRoundInput {
            round: rounds_per_epoch,
        }),
    }];
    let receipt = test_runner.execute_transaction(
        SystemTransaction {
            instructions,
            blobs: vec![],
            nonce: 0,
            pre_allocated_ids: BTreeSet::new(),
        }
        .get_executable(btreeset![AuthAddresses::validator_role()]),
    );

    // Assert
    let result = receipt.expect_commit(true);
    let next_epoch = result.next_epoch().expect("Should have next epoch");
    assert_eq!(next_epoch.1, initial_epoch + 1);
    assert!(!next_epoch.0.contains_key(&validator_address));
}

#[test]
fn registered_validator_with_stake_does_become_part_of_validator_on_epoch_change() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let num_unstake_epochs = 1u64;
    let genesis = create_genesis(
        BTreeMap::new(),
        BTreeMap::new(),
        initial_epoch,
        rounds_per_epoch,
        num_unstake_epochs,
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let (pub_key, _, account_address) = test_runner.new_account(false);
    let validator_address = test_runner.new_validator_with_pub_key(pub_key, account_address);
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account(account_address, VALIDATOR_OWNER_TOKEN)
        .withdraw_from_account(account_address, RADIX_TOKEN, Decimal::one())
        .register_validator(validator_address)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.stake_validator(validator_address, bucket_id)
        })
        .call_method(
            account_address,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );
    receipt.expect_commit_success();

    // Act
    let instructions = vec![Instruction::CallMethod {
        component_address: EPOCH_MANAGER,
        method_name: EPOCH_MANAGER_NEXT_ROUND_IDENT.to_string(),
        args: to_manifest_value(&EpochManagerNextRoundInput {
            round: rounds_per_epoch,
        }),
    }];
    let receipt = test_runner.execute_transaction(
        SystemTransaction {
            instructions,
            blobs: vec![],
            nonce: 0,
            pre_allocated_ids: BTreeSet::new(),
        }
        .get_executable(btreeset![AuthAddresses::validator_role()]),
    );

    // Assert
    let result = receipt.expect_commit(true);
    let next_epoch = result.next_epoch().expect("Should have next epoch");
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
    let num_unstake_epochs = 1u64;
    let validator_pub_key = EcdsaSecp256k1PrivateKey::from_u64(2u64)
        .unwrap()
        .public_key();
    let validator_account_address =
        ComponentAddress::virtual_account_from_public_key(&validator_pub_key);
    let mut validator_set_and_stake_owners = BTreeMap::new();
    validator_set_and_stake_owners.insert(
        validator_pub_key,
        (Decimal::one(), validator_account_address),
    );
    let genesis = create_genesis(
        validator_set_and_stake_owners,
        BTreeMap::new(),
        initial_epoch,
        rounds_per_epoch,
        num_unstake_epochs,
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let validator_address = test_runner.get_validator_with_key(&validator_pub_key);
    let manifest = ManifestBuilder::new()
        .create_proof_from_account(validator_account_address, VALIDATOR_OWNER_TOKEN)
        .lock_fee(test_runner.faucet_component(), 10.into())
        .unregister_validator(validator_address)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&validator_pub_key)],
    );
    receipt.expect_commit_success();

    // Act
    let instructions = vec![Instruction::CallMethod {
        component_address: EPOCH_MANAGER,
        method_name: EPOCH_MANAGER_NEXT_ROUND_IDENT.to_string(),
        args: to_manifest_value(&EpochManagerNextRoundInput {
            round: rounds_per_epoch,
        }),
    }];
    let receipt = test_runner.execute_transaction(
        SystemTransaction {
            instructions,
            blobs: vec![],
            nonce: 0,
            pre_allocated_ids: BTreeSet::new(),
        }
        .get_executable(btreeset![AuthAddresses::validator_role()]),
    );

    // Assert
    let result = receipt.expect_commit(true);
    let next_epoch = result.next_epoch().expect("Should have next epoch");
    assert_eq!(next_epoch.1, initial_epoch + 1);
    assert!(!next_epoch.0.contains_key(&validator_address));
}

#[test]
fn updated_validator_keys_gets_updated_on_epoch_change() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let num_unstake_epochs = 1u64;
    let validator_pub_key = EcdsaSecp256k1PrivateKey::from_u64(2u64)
        .unwrap()
        .public_key();
    let validator_account_address =
        ComponentAddress::virtual_account_from_public_key(&validator_pub_key);
    let mut validator_set_and_stake_owners = BTreeMap::new();
    validator_set_and_stake_owners.insert(
        validator_pub_key,
        (Decimal::one(), validator_account_address),
    );
    let genesis = create_genesis(
        validator_set_and_stake_owners,
        BTreeMap::new(),
        initial_epoch,
        rounds_per_epoch,
        num_unstake_epochs,
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let validator_address = test_runner.get_validator_with_key(&validator_pub_key);
    let next_validator_pub_key = EcdsaSecp256k1PrivateKey::from_u64(3u64)
        .unwrap()
        .public_key();
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account(validator_account_address, VALIDATOR_OWNER_TOKEN)
        .call_method(
            validator_address,
            "update_key",
            manifest_args!(next_validator_pub_key),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&validator_pub_key)],
    );
    receipt.expect_commit_success();

    // Act
    let instructions = vec![Instruction::CallMethod {
        component_address: EPOCH_MANAGER,
        method_name: EPOCH_MANAGER_NEXT_ROUND_IDENT.to_string(),
        args: to_manifest_value(&EpochManagerNextRoundInput {
            round: rounds_per_epoch,
        }),
    }];
    let receipt = test_runner.execute_transaction(
        SystemTransaction {
            instructions,
            blobs: vec![],
            nonce: 0,
            pre_allocated_ids: BTreeSet::new(),
        }
        .get_executable(btreeset![AuthAddresses::validator_role()]),
    );

    // Assert
    let result = receipt.expect_commit(true);
    let next_epoch = result.next_epoch().expect("Should have next epoch");
    assert_eq!(next_epoch.1, initial_epoch + 1);
    assert_eq!(
        next_epoch.0.get(&validator_address).unwrap().key,
        next_validator_pub_key
    );
}

#[test]
fn cannot_claim_unstake_immediately() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let num_unstake_epochs = 1u64;
    let validator_pub_key = EcdsaSecp256k1PrivateKey::from_u64(2u64)
        .unwrap()
        .public_key();
    let account_pub_key = EcdsaSecp256k1PrivateKey::from_u64(1u64)
        .unwrap()
        .public_key();
    let mut validator_set_and_stake_owners = BTreeMap::new();
    let account_with_lp = ComponentAddress::virtual_account_from_public_key(&account_pub_key);
    validator_set_and_stake_owners.insert(validator_pub_key, (Decimal::from(10), account_with_lp));
    let genesis = create_genesis(
        validator_set_and_stake_owners,
        BTreeMap::new(),
        initial_epoch,
        rounds_per_epoch,
        num_unstake_epochs,
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let validator_address = test_runner.get_validator_with_key(&validator_pub_key);
    let validator_substate = test_runner.get_validator_info(validator_address);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .withdraw_from_account(
            account_with_lp,
            validator_substate.liquidity_token,
            1.into(),
        )
        .take_from_worktop(validator_substate.liquidity_token, |builder, bucket| {
            builder.unstake_validator(validator_address, bucket)
        })
        .take_from_worktop(validator_substate.unstake_nft, |builder, bucket| {
            builder.claim_xrd(validator_address, bucket)
        })
        .call_method(
            account_with_lp,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_pub_key)],
    );

    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::ValidatorError(
                ValidatorError::EpochUnlockHasNotOccurredYet
            ))
        )
    });
}

#[test]
fn can_claim_unstake_after_epochs() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let num_unstake_epochs = 1u64;
    let validator_pub_key = EcdsaSecp256k1PrivateKey::from_u64(2u64)
        .unwrap()
        .public_key();
    let account_pub_key = EcdsaSecp256k1PrivateKey::from_u64(1u64)
        .unwrap()
        .public_key();
    let account_with_lp = ComponentAddress::virtual_account_from_public_key(&account_pub_key);
    let mut validator_set_and_stake_owners = BTreeMap::new();
    validator_set_and_stake_owners.insert(validator_pub_key, (Decimal::from(10), account_with_lp));
    let genesis = create_genesis(
        validator_set_and_stake_owners,
        BTreeMap::new(),
        initial_epoch,
        rounds_per_epoch,
        num_unstake_epochs,
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let validator_address = test_runner.get_validator_with_key(&validator_pub_key);
    let validator_substate = test_runner.get_validator_info(validator_address);
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .withdraw_from_account(
            account_with_lp,
            validator_substate.liquidity_token,
            1.into(),
        )
        .take_from_worktop(validator_substate.liquidity_token, |builder, bucket| {
            builder.unstake_validator(validator_address, bucket)
        })
        .call_method(
            account_with_lp,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_pub_key)],
    );
    receipt.expect_commit_success();
    test_runner.set_current_epoch(initial_epoch + 1 + num_unstake_epochs);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .withdraw_from_account(account_with_lp, validator_substate.unstake_nft, 1.into())
        .take_from_worktop(validator_substate.unstake_nft, |builder, bucket| {
            builder.claim_xrd(validator_address, bucket)
        })
        .call_method(
            account_with_lp,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_pub_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn unstaked_validator_gets_less_stake_on_epoch_change() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let num_unstake_epochs = 1u64;
    let validator_pub_key = EcdsaSecp256k1PrivateKey::from_u64(2u64)
        .unwrap()
        .public_key();
    let account_pub_key = EcdsaSecp256k1PrivateKey::from_u64(1u64)
        .unwrap()
        .public_key();
    let account_with_lp = ComponentAddress::virtual_account_from_public_key(&account_pub_key);
    let mut validator_set_and_stake_owners = BTreeMap::new();
    validator_set_and_stake_owners.insert(validator_pub_key, (Decimal::from(10), account_with_lp));
    let genesis = create_genesis(
        validator_set_and_stake_owners,
        BTreeMap::new(),
        initial_epoch,
        rounds_per_epoch,
        num_unstake_epochs,
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let validator_address = test_runner.get_validator_with_key(&validator_pub_key);
    let validator_substate = test_runner.get_validator_info(validator_address);
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .withdraw_from_account(
            account_with_lp,
            validator_substate.liquidity_token,
            Decimal::one(),
        )
        .take_from_worktop(validator_substate.liquidity_token, |builder, bucket| {
            builder.unstake_validator(validator_address, bucket)
        })
        .call_method(
            account_with_lp,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_pub_key)],
    );
    receipt.expect_commit_success();

    // Act
    let instructions = vec![Instruction::CallMethod {
        component_address: EPOCH_MANAGER,
        method_name: EPOCH_MANAGER_NEXT_ROUND_IDENT.to_string(),
        args: to_manifest_value(&EpochManagerNextRoundInput {
            round: rounds_per_epoch,
        }),
    }];
    let receipt = test_runner.execute_transaction(
        SystemTransaction {
            instructions,
            blobs: vec![],
            nonce: 0,
            pre_allocated_ids: BTreeSet::new(),
        }
        .get_executable(btreeset![AuthAddresses::validator_role()]),
    );

    // Assert
    let result = receipt.expect_commit(true);
    let next_epoch = result.next_epoch().expect("Should have next epoch");
    assert_eq!(next_epoch.1, initial_epoch + 1);
    assert_eq!(
        next_epoch.0.get(&validator_address).unwrap(),
        &Validator {
            key: validator_pub_key,
            stake: Decimal::from(9),
        }
    );
}

#[test]
fn epoch_manager_create_should_fail_with_supervisor_privilege() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let mut pre_allocated_ids = BTreeSet::new();
    pre_allocated_ids.insert(EPOCH_MANAGER.into());
    pre_allocated_ids.insert(VALIDATOR_OWNER_TOKEN.into());
    let validator_set: BTreeMap<EcdsaSecp256k1PublicKey, ManifestValidatorInit> = BTreeMap::new();
    let instructions = vec![Instruction::CallFunction {
        package_address: EPOCH_MANAGER_PACKAGE,
        blueprint_name: EPOCH_MANAGER_BLUEPRINT.to_string(),
        function_name: EPOCH_MANAGER_CREATE_IDENT.to_string(),
        args: manifest_args!(
            Into::<[u8; 27]>::into(VALIDATOR_OWNER_TOKEN),
            Into::<[u8; 27]>::into(EPOCH_MANAGER),
            validator_set,
            1u64,
            1u64,
            1u64
        ),
    }];
    let blobs = vec![];
    let receipt = test_runner.execute_transaction(
        SystemTransaction {
            instructions,
            blobs,
            nonce: 0,
            pre_allocated_ids,
        }
        .get_executable(btreeset![]),
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError { .. }))
    });
}

#[test]
fn epoch_manager_create_should_succeed_with_system_privilege() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let mut pre_allocated_ids = BTreeSet::new();
    pre_allocated_ids.insert(EPOCH_MANAGER.into());
    pre_allocated_ids.insert(VALIDATOR_OWNER_TOKEN.into());

    let validator_set: BTreeMap<EcdsaSecp256k1PublicKey, ManifestValidatorInit> = BTreeMap::new();
    let instructions = vec![Instruction::CallFunction {
        package_address: EPOCH_MANAGER_PACKAGE,
        blueprint_name: EPOCH_MANAGER_BLUEPRINT.to_string(),
        function_name: "create".to_string(),
        args: manifest_args!(
            Into::<[u8; 27]>::into(VALIDATOR_OWNER_TOKEN),
            Into::<[u8; 27]>::into(EPOCH_MANAGER),
            validator_set,
            1u64,
            1u64,
            1u64
        ),
    }];
    let blobs = vec![];
    let receipt = test_runner.execute_transaction(
        SystemTransaction {
            instructions,
            blobs,
            nonce: 0,
            pre_allocated_ids,
        }
        .get_executable(btreeset![AuthAddresses::system_role()]),
    );

    // Assert
    receipt.expect_commit_success();
}
