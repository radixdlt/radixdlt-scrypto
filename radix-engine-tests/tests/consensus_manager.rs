use radix_engine::blueprints::consensus_manager::{
    Validator, ValidatorEmissionAppliedEvent, ValidatorError,
};
use radix_engine::errors::{ApplicationError, ModuleError, RuntimeError};
use radix_engine::system::bootstrap::*;
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::blueprints::resource::FromPublicKey;
use rand::prelude::SliceRandom;
use rand::Rng;
use rand_chacha;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;
use scrypto_unit::*;
use transaction::builder::{ManifestBuilder, TransactionManifestV1};
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
use transaction::model::InstructionV1;

#[test]
fn genesis_epoch_has_correct_initial_validators() {
    // Arrange
    let initial_epoch = 1u64;
    let max_validators = 100u32;

    let mut stake_allocations = Vec::new();
    let mut validators = Vec::new();
    let mut accounts = Vec::new();
    let mut keys = BTreeMap::<EcdsaSecp256k1PublicKey, usize>::new();
    for k in 1usize..=150usize {
        let pub_key = EcdsaSecp256k1PrivateKey::from_u64(k.try_into().unwrap())
            .unwrap()
            .public_key();
        keys.insert(pub_key.clone(), k);
        let validator_account_address = ComponentAddress::virtual_account_from_public_key(&pub_key);
        accounts.push(validator_account_address);
        validators.push(GenesisValidator {
            key: pub_key,
            accept_delegated_stake: true,
            is_registered: true,
            metadata: vec![],
            owner: validator_account_address,
        });

        let stake = if k == 91 {
            Decimal::from(1000000 * 1000)
        } else if k == 104 {
            Decimal::from(1000000 * 990)
        } else if k <= 10 {
            Decimal::from(100000) // All the same
        } else if k <= 100 {
            Decimal::from(1000000 * ((k + 1) / 2))
        } else {
            Decimal::from(0)
        };

        stake_allocations.push((
            pub_key,
            vec![GenesisStakeAllocation {
                account_index: (k - 1) as u32,
                xrd_amount: stake,
            }],
        ));
    }

    let genesis_data_chunks = vec![
        GenesisDataChunk::Validators(validators),
        GenesisDataChunk::Stakes {
            accounts,
            allocations: stake_allocations,
        },
    ];

    let genesis = CustomGenesis {
        genesis_data_chunks,
        initial_epoch,
        initial_configuration: dummy_consensus_manager_configuration()
            .with_max_validators(max_validators),
        initial_time_ms: 1,
    };

    // Act
    let (_, validator_set) = TestRunner::builder()
        .with_custom_genesis(genesis)
        .build_and_get_epoch();

    // Assert
    assert_eq!(validator_set.validator_count(), max_validators as usize);

    for (i, (_, validator)) in validator_set
        .validators_by_stake_desc
        .into_iter()
        .enumerate()
    {
        let index = *keys.get(&validator.key).unwrap();
        // Check that the validator set is in order stake DESC
        // Based on the weird special-casing of certain validators we defined above
        match i {
            0 => {
                assert_eq!(index, 91);
                assert_eq!(validator.stake, Decimal::from(1000000 * 1000));
            }
            1 => {
                assert_eq!(index, 104);
                assert_eq!(validator.stake, Decimal::from(1000000 * 990));
            }
            x if x < 91 => {
                assert!(index > 10);
                assert!(validator.stake >= Decimal::from(5000000));
                assert!(validator.stake <= Decimal::from(50000000));
            }
            _ => {
                assert!(index <= 10);
                assert_eq!(validator.stake, Decimal::from(100000));
            }
        }
    }
}

#[test]
fn get_epoch_should_succeed() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/consensus_manager");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "ConsensusManagerTest",
            "get_epoch",
            manifest_args![],
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let epoch: u64 = receipt.expect_commit_success().output(1);
    assert_eq!(epoch, 1);
}

#[test]
fn next_round_without_supervisor_auth_fails() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/consensus_manager");

    // Act
    let round = 9876u64;
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "ConsensusManagerTest",
            "next_round",
            manifest_args!(CONSENSUS_MANAGER, round),
        )
        .call_function(
            package_address,
            "ConsensusManagerTest",
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
    let initial_epoch = 1u64;
    let rounds_per_epoch = 5u64;
    let genesis = CustomGenesis::default(
        initial_epoch,
        dummy_consensus_manager_configuration().with_rounds_per_epoch(rounds_per_epoch),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();

    // Act
    let receipt = test_runner.advance_to_round(rounds_per_epoch - 1);

    // Assert
    let result = receipt.expect_commit_success();
    assert!(result.next_epoch().is_none());
}

#[test]
fn next_epoch_with_validator_auth_succeeds() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let genesis = CustomGenesis::default(
        initial_epoch,
        dummy_consensus_manager_configuration().with_rounds_per_epoch(rounds_per_epoch),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();

    // Act
    let receipt = test_runner.advance_to_round(rounds_per_epoch);

    // Assert
    let result = receipt.expect_commit_success();
    let next_epoch = result.next_epoch().expect("Should have next epoch").epoch;
    assert_eq!(next_epoch, initial_epoch + 1);
}

#[test]
fn register_validator_with_auth_succeeds() {
    // Arrange
    let initial_epoch = 5u64;
    let pub_key = EcdsaSecp256k1PrivateKey::from_u64(1u64)
        .unwrap()
        .public_key();
    let validator_account_address = ComponentAddress::virtual_account_from_public_key(&pub_key);
    let genesis = CustomGenesis::single_validator_and_staker(
        pub_key,
        Decimal::one(),
        validator_account_address,
        initial_epoch,
        dummy_consensus_manager_configuration(),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();

    // Act
    let validator_address = test_runner.get_active_validator_with_key(&pub_key);
    let manifest = ManifestBuilder::new()
        .create_proof_from_account(validator_account_address, VALIDATOR_OWNER_BADGE)
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
    let pub_key = EcdsaSecp256k1PrivateKey::from_u64(1u64)
        .unwrap()
        .public_key();
    let validator_account_address = ComponentAddress::virtual_account_from_public_key(&pub_key);
    let genesis = CustomGenesis::single_validator_and_staker(
        pub_key,
        Decimal::one(),
        validator_account_address,
        initial_epoch,
        dummy_consensus_manager_configuration(),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();

    // Act
    let validator_address = test_runner.get_active_validator_with_key(&pub_key);
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
    let pub_key = EcdsaSecp256k1PrivateKey::from_u64(1u64)
        .unwrap()
        .public_key();
    let validator_account_address = ComponentAddress::virtual_account_from_public_key(&pub_key);
    let genesis = CustomGenesis::single_validator_and_staker(
        pub_key,
        Decimal::one(),
        validator_account_address,
        initial_epoch,
        dummy_consensus_manager_configuration(),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();

    // Act
    let validator_address = test_runner.get_active_validator_with_key(&pub_key);
    let manifest = ManifestBuilder::new()
        .create_proof_from_account(validator_account_address, VALIDATOR_OWNER_BADGE)
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
    let pub_key = EcdsaSecp256k1PrivateKey::from_u64(1u64)
        .unwrap()
        .public_key();
    let validator_account_address = ComponentAddress::virtual_account_from_public_key(&pub_key);
    let genesis = CustomGenesis::single_validator_and_staker(
        pub_key,
        Decimal::one(),
        validator_account_address,
        initial_epoch,
        dummy_consensus_manager_configuration(),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();

    // Act
    let validator_address = test_runner.get_active_validator_with_key(&pub_key);
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
    let pub_key = EcdsaSecp256k1PrivateKey::from_u64(1u64)
        .unwrap()
        .public_key();
    let validator_account_address = ComponentAddress::virtual_account_from_public_key(&pub_key);
    let genesis = CustomGenesis::single_validator_and_staker(
        pub_key,
        Decimal::one(),
        validator_account_address,
        initial_epoch,
        dummy_consensus_manager_configuration(),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let validator_address = test_runner.get_active_validator_with_key(&pub_key);
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account(validator_account_address, VALIDATOR_OWNER_BADGE)
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
        builder.create_proof_from_account(validator_account_address, VALIDATOR_OWNER_BADGE);
    }

    let manifest = builder
        .call_method(test_runner.faucet_component(), "free", manifest_args!())
        .take_all_from_worktop(RADIX_TOKEN, |builder, bucket| {
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
fn registered_validator_with_no_stake_does_not_become_part_of_validator_set_on_epoch_change() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let genesis = CustomGenesis::default(
        initial_epoch,
        dummy_consensus_manager_configuration().with_rounds_per_epoch(rounds_per_epoch),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let (pub_key, _, account_address) = test_runner.new_account(false);
    let validator_address = test_runner.new_validator_with_pub_key(pub_key, account_address);
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account(account_address, VALIDATOR_OWNER_BADGE)
        .register_validator(validator_address)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );
    receipt.expect_commit_success();

    // Act
    let receipt = test_runner.advance_to_round(rounds_per_epoch);

    // Assert
    let result = receipt.expect_commit_success();
    let next_epoch = result.next_epoch().expect("Should have next epoch");
    assert_eq!(next_epoch.epoch, initial_epoch + 1);
    assert!(!next_epoch
        .validator_set
        .get_by_address(&validator_address)
        .is_some());
}

#[test]
fn validator_set_receives_emissions_proportional_to_stake_on_epoch_change() {
    // Arrange
    let initial_epoch = 2;
    let epoch_emissions_xrd = dec!("0.1");
    let a_initial_stake = dec!("2.5");
    let b_initial_stake = dec!("7.5");
    let both_initial_stake = a_initial_stake + b_initial_stake;

    let a_key = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key();
    let b_key = EcdsaSecp256k1PrivateKey::from_u64(2).unwrap().public_key();
    let validators = vec![GenesisValidator::from(a_key), GenesisValidator::from(b_key)];
    let allocations = vec![
        (
            a_key,
            vec![GenesisStakeAllocation {
                account_index: 0,
                xrd_amount: a_initial_stake,
            }],
        ),
        (
            b_key,
            vec![GenesisStakeAllocation {
                account_index: 1,
                xrd_amount: b_initial_stake,
            }],
        ),
    ];
    let accounts = validators
        .iter()
        .map(|validator| validator.owner)
        .collect::<Vec<_>>();
    let genesis_data_chunks = vec![
        GenesisDataChunk::Validators(validators),
        GenesisDataChunk::Stakes {
            accounts,
            allocations,
        },
    ];
    let genesis = CustomGenesis {
        genesis_data_chunks,
        initial_epoch,
        initial_configuration: dummy_consensus_manager_configuration()
            .with_rounds_per_epoch(1)
            .with_total_emission_xrd_per_epoch(epoch_emissions_xrd),
        initial_time_ms: 1,
    };

    // Act
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let receipt = test_runner.advance_to_round(1);

    // Assert
    let a_substate = test_runner.get_validator_info_by_key(&a_key);
    let a_new_stake = test_runner
        .inspect_vault_balance(a_substate.stake_xrd_vault_id.0)
        .unwrap();
    let a_stake_added = epoch_emissions_xrd * a_initial_stake / both_initial_stake;
    assert_eq!(a_new_stake, a_initial_stake + a_stake_added);

    let b_substate = test_runner.get_validator_info_by_key(&b_key);
    let b_new_stake = test_runner
        .inspect_vault_balance(b_substate.stake_xrd_vault_id.0)
        .unwrap();
    let b_stake_added = epoch_emissions_xrd * b_initial_stake / both_initial_stake;
    assert_eq!(b_new_stake, b_initial_stake + b_stake_added);

    let result = receipt.expect_commit_success();
    let next_epoch_validators = result
        .next_epoch()
        .expect("Should have next epoch")
        .validator_set
        .validators_by_stake_desc
        .into_values()
        .collect::<Vec<_>>();

    assert!(b_new_stake > a_new_stake);
    assert_eq!(
        next_epoch_validators,
        // Note - it's ordered by stake desc, so b is first
        vec![
            Validator {
                key: b_key,
                stake: b_new_stake,
            },
            Validator {
                key: a_key,
                stake: a_new_stake,
            },
        ]
    );

    let emission_applied_events = result
        .application_events
        .iter()
        .filter(|(id, _data)| test_runner.is_event_name_equal::<ValidatorEmissionAppliedEvent>(id))
        .map(|(id, data)| {
            (
                extract_emitter_node_id(id),
                scrypto_decode::<ValidatorEmissionAppliedEvent>(data).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        emission_applied_events,
        // Note - emissions are output in the order of the active validator set, so b is first as it has higher stake
        vec![
            (
                test_runner
                    .get_active_validator_with_key(&b_key)
                    .into_node_id(),
                ValidatorEmissionAppliedEvent {
                    epoch: initial_epoch,
                    starting_stake_pool_xrd: b_initial_stake,
                    stake_pool_added_xrd: Decimal::zero(), // default `fee_factor = 1.0` zeroes out the net emission for regular stakers
                    total_stake_unit_supply: b_initial_stake, // stays at the level captured before any emissions
                    validator_fee_xrd: b_stake_added, // default `fee_factor = 1.0` takes the entire emission as fee
                    proposals_made: 1,
                    proposals_missed: 0,
                }
            ),
            (
                test_runner
                    .get_active_validator_with_key(&a_key)
                    .into_node_id(),
                ValidatorEmissionAppliedEvent {
                    epoch: initial_epoch,
                    starting_stake_pool_xrd: a_initial_stake,
                    stake_pool_added_xrd: Decimal::zero(), // default `fee_factor = 1.0` zeroes out the net emission for regular stakers
                    total_stake_unit_supply: a_initial_stake, // stays at the level captured before any emissions
                    validator_fee_xrd: a_stake_added, // default `fee_factor = 1.0` takes the entire emission as fee
                    proposals_made: 0,
                    proposals_missed: 0,
                }
            ),
        ]
    );
}

#[test]
fn validator_receives_emission_penalty_when_some_proposals_missed() {
    // Arrange
    let initial_epoch = 5;
    let epoch_emissions_xrd = dec!("10");
    let rounds_per_epoch = 4; // we will simulate 3 gap rounds + 1 successfully made proposal...
    let min_required_reliability = dec!("0.2"); // ...which barely meets the threshold
    let validator_pub_key = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key();
    let validator_initial_stake = dec!("500.0");
    let genesis = CustomGenesis::single_validator_and_staker(
        validator_pub_key,
        validator_initial_stake,
        ComponentAddress::virtual_account_from_public_key(&validator_pub_key),
        initial_epoch,
        dummy_consensus_manager_configuration()
            .with_rounds_per_epoch(rounds_per_epoch)
            .with_total_emission_xrd_per_epoch(epoch_emissions_xrd)
            .with_min_validator_reliability(min_required_reliability),
    );

    // Act
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let receipt = test_runner.advance_to_round(rounds_per_epoch);

    // Assert: stake vault balance increased by the given emission * reliability factor
    let validator_substate = test_runner.get_validator_info_by_key(&validator_pub_key);
    let validator_new_stake = test_runner
        .inspect_vault_balance(validator_substate.stake_xrd_vault_id.0)
        .unwrap();
    let actual_reliability = Decimal::one() / Decimal::from(rounds_per_epoch);
    let tolerated_range = Decimal::one() - min_required_reliability;
    let reliability_factor = (actual_reliability - min_required_reliability) / tolerated_range;
    let validator_stake_added = epoch_emissions_xrd * reliability_factor;
    assert_eq!(
        validator_new_stake,
        validator_initial_stake + validator_stake_added
    );

    // Assert: owner stake vault balance increased by that same number (because of default `fee_factor = 1.0`)
    // Note: we know this number because an exchange rate of stake units is 1:1 (during the first epoch!)
    assert_eq!(
        test_runner.inspect_vault_balance(validator_substate.locked_owner_stake_unit_vault_id.0),
        Some(validator_stake_added)
    );

    // Assert: the next epoch event reflects the new amount of staked XRD for this validator
    let result = receipt.expect_commit_success();
    let next_epoch_validators = result
        .next_epoch()
        .expect("Should have next epoch")
        .validator_set
        .validators_by_stake_desc
        .into_values()
        .collect::<Vec<_>>();
    assert_eq!(
        next_epoch_validators,
        vec![Validator {
            key: validator_pub_key,
            stake: validator_new_stake,
        },]
    );

    // Assert: emitted event gives the details/breakdown
    assert_eq!(
        test_runner.extract_events_of_type::<ValidatorEmissionAppliedEvent>(result),
        vec![ValidatorEmissionAppliedEvent {
            epoch: initial_epoch,
            starting_stake_pool_xrd: validator_initial_stake,
            stake_pool_added_xrd: Decimal::zero(), // default `fee_factor = 1.0` zeroes out the net emission for regular stakers
            total_stake_unit_supply: validator_initial_stake, // stays at the level captured before any emissions
            validator_fee_xrd: validator_stake_added, // default `fee_factor = 1.0` takes the entire emission as fee
            proposals_made: 1,
            proposals_missed: 3,
        },]
    );
}

#[test]
fn validator_receives_no_emission_when_too_many_proposals_missed() {
    // Arrange
    let initial_epoch = 7;
    let epoch_emissions_xrd = dec!("10");
    let rounds_per_epoch = 4; // we will simulate 3 gap rounds + 1 successfully made proposal...
    let min_required_reliability = dec!("0.3"); // ...which does NOT meet the threshold
    let validator_pub_key = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key();
    let validator_stake = dec!("500.0");
    let genesis = CustomGenesis::single_validator_and_staker(
        validator_pub_key,
        validator_stake,
        ComponentAddress::virtual_account_from_public_key(&validator_pub_key),
        initial_epoch,
        dummy_consensus_manager_configuration()
            .with_rounds_per_epoch(rounds_per_epoch)
            .with_total_emission_xrd_per_epoch(epoch_emissions_xrd)
            .with_min_validator_reliability(min_required_reliability),
    );

    // Act
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let receipt = test_runner.advance_to_round(rounds_per_epoch);

    // Assert
    let validator_substate = test_runner.get_validator_info_by_key(&validator_pub_key);
    let validator_new_stake = test_runner
        .inspect_vault_balance(validator_substate.stake_xrd_vault_id.0)
        .unwrap();
    assert_eq!(validator_new_stake, validator_stake);

    let result = receipt.expect_commit_success();
    let next_epoch_validators = result
        .next_epoch()
        .expect("Should have next epoch")
        .validator_set
        .validators_by_stake_desc
        .into_values()
        .collect::<Vec<_>>();
    assert_eq!(
        next_epoch_validators,
        vec![Validator {
            key: validator_pub_key,
            stake: validator_stake
        },]
    );

    assert_eq!(
        test_runner.extract_events_of_type::<ValidatorEmissionAppliedEvent>(result),
        vec![ValidatorEmissionAppliedEvent {
            epoch: initial_epoch,
            starting_stake_pool_xrd: validator_stake,
            stake_pool_added_xrd: Decimal::zero(), // even though the emission gave 0 XRD to the regular stakers...
            total_stake_unit_supply: validator_stake,
            validator_fee_xrd: Decimal::zero(), // ... or to the owner...
            proposals_made: 1,
            proposals_missed: 3, // ... we still want this event, e.g. to surface this information
        },]
    );
}

#[test]
fn decreasing_validator_fee_takes_effect_during_next_epoch() {
    // Arrange
    let initial_epoch = 7;
    let initial_stake_amount = dec!("4000.0"); // big and round numbers
    let emission_xrd_per_epoch = dec!("1000.0"); // to avoid rounding errors
    let next_epoch_fee_factor = dec!("0.25"); // for easier asserts
    let validator_key = EcdsaSecp256k1PrivateKey::from_u64(2u64)
        .unwrap()
        .public_key();
    let validator_account = ComponentAddress::virtual_account_from_public_key(&validator_key);
    let genesis = CustomGenesis::single_validator_and_staker(
        validator_key,
        initial_stake_amount,
        validator_account,
        initial_epoch,
        dummy_consensus_manager_configuration()
            .with_total_emission_xrd_per_epoch(emission_xrd_per_epoch)
            .with_rounds_per_epoch(1), // deliberate, to go through rounds/epoch without gaps
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let validator_address = test_runner.get_active_validator_with_key(&validator_key);

    // Act: request the fee decrease
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account(validator_account, VALIDATOR_OWNER_BADGE)
        .call_method(
            validator_address,
            VALIDATOR_UPDATE_FEE_IDENT,
            manifest_args!(next_epoch_fee_factor),
        )
        .build();
    test_runner
        .execute_manifest(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&validator_key)],
        )
        .expect_commit_success();

    // Act: change epoch
    let receipt = test_runner.advance_to_round(1);

    // Assert: no change yet (the default `fee_factor = 1.0` was effective during that epoch)
    let result = receipt.expect_commit_success();
    assert_eq!(
        test_runner.extract_events_of_type::<ValidatorEmissionAppliedEvent>(result),
        vec![ValidatorEmissionAppliedEvent {
            epoch: initial_epoch,
            starting_stake_pool_xrd: initial_stake_amount,
            stake_pool_added_xrd: Decimal::zero(), // default `fee_factor = 1.0` zeroes out the net emission for regular stakers
            total_stake_unit_supply: initial_stake_amount, // stays at the level captured before any emissions
            validator_fee_xrd: emission_xrd_per_epoch, // default `fee_factor = 1.0` takes the entire emission as fee
            proposals_made: 1,
            proposals_missed: 0,
        },]
    );
    let first_epoch_fee_xrd = emission_xrd_per_epoch; // the entire emission was raked...
    let validator_substate = test_runner.get_validator_info_by_key(&validator_key);
    assert_eq!(
        test_runner.inspect_vault_balance(validator_substate.stake_xrd_vault_id.0),
        Some(initial_stake_amount + first_epoch_fee_xrd) // ... and it also was auto-staked...
    );
    assert_eq!(
        test_runner.inspect_vault_balance(validator_substate.locked_owner_stake_unit_vault_id.0),
        Some(first_epoch_fee_xrd) // ... and locked in the internal owner's vault
    );

    // Act: change epoch
    let receipt = test_runner.advance_to_round(1);

    // Assert: during that next epoch, the `next_epoch_fee_factor` was already effective
    let result = receipt.expect_commit_success();
    let next_epoch_start_stake_xrd = initial_stake_amount + emission_xrd_per_epoch;
    let next_epoch_fee_xrd = emission_xrd_per_epoch * next_epoch_fee_factor;
    let next_epoch_net_emission_xrd = emission_xrd_per_epoch - next_epoch_fee_xrd;
    assert_eq!(
        test_runner.extract_events_of_type::<ValidatorEmissionAppliedEvent>(result),
        vec![ValidatorEmissionAppliedEvent {
            epoch: initial_epoch + 1,
            starting_stake_pool_xrd: next_epoch_start_stake_xrd,
            stake_pool_added_xrd: next_epoch_net_emission_xrd,
            total_stake_unit_supply: next_epoch_start_stake_xrd, // we auto-staked 100%, so the rate is still 1:1
            validator_fee_xrd: next_epoch_fee_xrd,
            proposals_made: 1,
            proposals_missed: 0,
        },]
    );
    let validator_substate = test_runner.get_validator_info_by_key(&validator_key);
    assert_eq!(
        test_runner.inspect_vault_balance(validator_substate.stake_xrd_vault_id.0),
        Some(initial_stake_amount + dec!("2") * emission_xrd_per_epoch) // everything still goes into stake, by various means
    );
    // the new fee goes into internal owner's vault (as stake units)
    let stake_unit_exchange_rate =
        next_epoch_start_stake_xrd / (next_epoch_start_stake_xrd + next_epoch_net_emission_xrd);
    assert_eq!(
        precise_enough(
            test_runner
                .inspect_vault_balance(validator_substate.locked_owner_stake_unit_vault_id.0)
                .unwrap()
        ),
        precise_enough(first_epoch_fee_xrd + next_epoch_fee_xrd * stake_unit_exchange_rate)
    );
}

#[test]
fn increasing_validator_fee_takes_effect_after_configured_epochs_delay() {
    // Arrange
    let initial_epoch = 7;
    let fee_increase_delay_epochs = 4;
    let initial_stake_amount = dec!("9.0");
    let emission_xrd_per_epoch = dec!("2.0");
    let increased_fee_factor = dec!("0.25");
    let validator_key = EcdsaSecp256k1PrivateKey::from_u64(2u64)
        .unwrap()
        .public_key();
    let validator_account = ComponentAddress::virtual_account_from_public_key(&validator_key);
    let genesis = CustomGenesis::single_validator_and_staker(
        validator_key,
        initial_stake_amount,
        validator_account,
        initial_epoch,
        dummy_consensus_manager_configuration()
            .with_total_emission_xrd_per_epoch(emission_xrd_per_epoch)
            .with_num_fee_increase_delay_epochs(fee_increase_delay_epochs)
            .with_rounds_per_epoch(1), // deliberate, to go through rounds/epoch without gaps
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let validator_address = test_runner.get_active_validator_with_key(&validator_key);
    let stake_xrd_vault_id = test_runner
        .get_validator_info(validator_address)
        .stake_xrd_vault_id
        .0;

    // we have to first request some fee decrease...
    test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee(test_runner.faucet_component(), 10.into())
                .create_proof_from_account(validator_account, VALIDATOR_OWNER_BADGE)
                .call_method(
                    validator_address,
                    VALIDATOR_UPDATE_FEE_IDENT,
                    manifest_args!(Decimal::zero()),
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&validator_key)],
        )
        .expect_commit_success();

    // ... and wait 1 epoch to make it effective
    test_runner.advance_to_round(1).expect_commit_success();
    let current_epoch = initial_epoch + 1;

    // Act: request the fee increase
    test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee(test_runner.faucet_component(), 10.into())
                .create_proof_from_account(validator_account, VALIDATOR_OWNER_BADGE)
                .call_method(
                    validator_address,
                    VALIDATOR_UPDATE_FEE_IDENT,
                    manifest_args!(increased_fee_factor),
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&validator_key)],
        )
        .expect_commit_success();
    let increase_effective_at_epoch = current_epoch + fee_increase_delay_epochs;

    // advance a few epochs (just 1 short of the increase being effective)
    for _ in current_epoch..increase_effective_at_epoch {
        test_runner.advance_to_round(1).expect_commit_success();
    }

    // Assert: no change yet (the default `fee_factor = 1.0` was effective during all these epochs)
    let num_epochs_with_default_fee = Decimal::from(increase_effective_at_epoch - initial_epoch);
    assert_eq!(
        precise_enough(
            test_runner
                .inspect_vault_balance(stake_xrd_vault_id)
                .unwrap()
        ),
        precise_enough(initial_stake_amount + num_epochs_with_default_fee * emission_xrd_per_epoch)
    );

    // Act: advance one more epoch
    let receipt = test_runner.advance_to_round(1);

    // Assert: during that next epoch, the `increased_fee_factor` was already effective
    let result = receipt.expect_commit_success();
    let event = test_runner
        .extract_events_of_type::<ValidatorEmissionAppliedEvent>(result)
        .remove(0);
    let rounded_amounts_event = ValidatorEmissionAppliedEvent {
        epoch: event.epoch,
        starting_stake_pool_xrd: precise_enough(event.starting_stake_pool_xrd),
        stake_pool_added_xrd: precise_enough(event.stake_pool_added_xrd),
        total_stake_unit_supply: precise_enough(event.total_stake_unit_supply),
        validator_fee_xrd: precise_enough(event.validator_fee_xrd),
        proposals_made: event.proposals_made,
        proposals_missed: event.proposals_missed,
    };
    assert_eq!(
        rounded_amounts_event,
        ValidatorEmissionAppliedEvent {
            epoch: increase_effective_at_epoch,
            starting_stake_pool_xrd: precise_enough(
                initial_stake_amount + num_epochs_with_default_fee * emission_xrd_per_epoch
            ),
            stake_pool_added_xrd: precise_enough(
                emission_xrd_per_epoch * (Decimal::one() - increased_fee_factor)
            ),
            // note: we staked (i.e. minted stake units) only during the first round; later a fee of 0% was effective, until now
            total_stake_unit_supply: precise_enough(initial_stake_amount + emission_xrd_per_epoch),
            validator_fee_xrd: precise_enough(emission_xrd_per_epoch * increased_fee_factor),
            proposals_made: 1,
            proposals_missed: 0,
        }
    );
}

fn create_custom_genesis(
    initial_epoch: u64,
    rounds_per_epoch: u64,
    num_initial_validators: usize,
    max_validators: usize,
    initial_stakes: Decimal,
    accounts_xrd_balance: Decimal,
    num_accounts: usize,
) -> (
    CustomGenesis,
    Vec<(EcdsaSecp256k1PublicKey, ComponentAddress)>,
) {
    let mut stake_allocations = Vec::new();
    let mut validators = Vec::new();
    let mut accounts = Vec::new();
    for k in 1usize..=num_initial_validators {
        let pub_key = EcdsaSecp256k1PrivateKey::from_u64(k.try_into().unwrap())
            .unwrap()
            .public_key();
        let validator_account_address = ComponentAddress::virtual_account_from_public_key(&pub_key);

        accounts.push(validator_account_address);
        validators.push(GenesisValidator {
            key: pub_key,
            accept_delegated_stake: true,
            is_registered: true,
            metadata: vec![],
            owner: validator_account_address,
        });

        stake_allocations.push((
            pub_key,
            vec![GenesisStakeAllocation {
                account_index: (k - 1) as u32,
                xrd_amount: initial_stakes,
            }],
        ));
    }

    let validator_account_index = num_initial_validators;

    let mut xrd_balances = Vec::new();
    let mut pub_key_accounts = Vec::new();

    for i in 0..num_accounts {
        let pub_key = EcdsaSecp256k1PrivateKey::from_u64(
            (validator_account_index + 1 + i).try_into().unwrap(),
        )
        .unwrap()
        .public_key();
        let account_address = ComponentAddress::virtual_account_from_public_key(&pub_key);
        pub_key_accounts.push((pub_key, account_address));
        xrd_balances.push((account_address, accounts_xrd_balance));
    }

    let genesis_data_chunks = vec![
        GenesisDataChunk::Validators(validators),
        GenesisDataChunk::Stakes {
            accounts,
            allocations: stake_allocations,
        },
        GenesisDataChunk::XrdBalances(xrd_balances),
    ];

    let genesis = CustomGenesis {
        genesis_data_chunks,
        initial_epoch,
        initial_configuration: dummy_consensus_manager_configuration()
            .with_max_validators(max_validators as u32)
            .with_rounds_per_epoch(rounds_per_epoch),
        initial_time_ms: 1,
    };

    (genesis, pub_key_accounts)
}

#[derive(Clone, Copy)]
enum RegisterAndStakeTransactionType {
    SingleManifestRegisterFirst,
    SingleManifestStakeFirst,
    RegisterFirst,
    StakeFirst,
}

impl RegisterAndStakeTransactionType {
    const ALL_TYPES: [RegisterAndStakeTransactionType; 4] = [
        RegisterAndStakeTransactionType::SingleManifestStakeFirst,
        RegisterAndStakeTransactionType::SingleManifestRegisterFirst,
        RegisterAndStakeTransactionType::RegisterFirst,
        RegisterAndStakeTransactionType::StakeFirst,
    ];

    fn manifests(
        &self,
        stake_amount: Decimal,
        account_address: ComponentAddress,
        validator_address: ComponentAddress,
        faucet: GlobalAddress,
    ) -> Vec<TransactionManifestV1> {
        match self {
            RegisterAndStakeTransactionType::SingleManifestRegisterFirst => {
                let manifest = ManifestBuilder::new()
                    .lock_fee(faucet, 10.into())
                    .create_proof_from_account(account_address, VALIDATOR_OWNER_BADGE)
                    .withdraw_from_account(account_address, RADIX_TOKEN, stake_amount)
                    .register_validator(validator_address)
                    .take_all_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                        builder.stake_validator(validator_address, bucket_id)
                    })
                    .call_method(
                        account_address,
                        "deposit_batch",
                        manifest_args!(ManifestExpression::EntireWorktop),
                    )
                    .build();
                vec![manifest]
            }
            RegisterAndStakeTransactionType::SingleManifestStakeFirst => {
                let manifest = ManifestBuilder::new()
                    .lock_fee(faucet, 10.into())
                    .create_proof_from_account(account_address, VALIDATOR_OWNER_BADGE)
                    .withdraw_from_account(account_address, RADIX_TOKEN, stake_amount)
                    .take_all_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                        builder.stake_validator(validator_address, bucket_id)
                    })
                    .register_validator(validator_address)
                    .call_method(
                        account_address,
                        "deposit_batch",
                        manifest_args!(ManifestExpression::EntireWorktop),
                    )
                    .build();
                vec![manifest]
            }
            RegisterAndStakeTransactionType::RegisterFirst => {
                let register_manifest = ManifestBuilder::new()
                    .lock_fee(faucet, 10.into())
                    .create_proof_from_account(account_address, VALIDATOR_OWNER_BADGE)
                    .register_validator(validator_address)
                    .build();

                let stake_manifest = ManifestBuilder::new()
                    .lock_fee(faucet, 10.into())
                    .create_proof_from_account(account_address, VALIDATOR_OWNER_BADGE)
                    .withdraw_from_account(account_address, RADIX_TOKEN, stake_amount)
                    .take_all_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                        builder.stake_validator(validator_address, bucket_id)
                    })
                    .call_method(
                        account_address,
                        "deposit_batch",
                        manifest_args!(ManifestExpression::EntireWorktop),
                    )
                    .build();

                vec![register_manifest, stake_manifest]
            }
            RegisterAndStakeTransactionType::StakeFirst => {
                let register_manifest = ManifestBuilder::new()
                    .lock_fee(faucet, 10.into())
                    .create_proof_from_account(account_address, VALIDATOR_OWNER_BADGE)
                    .register_validator(validator_address)
                    .build();

                let stake_manifest = ManifestBuilder::new()
                    .lock_fee(faucet, 10.into())
                    .create_proof_from_account(account_address, VALIDATOR_OWNER_BADGE)
                    .withdraw_from_account(account_address, RADIX_TOKEN, stake_amount)
                    .take_all_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                        builder.stake_validator(validator_address, bucket_id)
                    })
                    .call_method(
                        account_address,
                        "deposit_batch",
                        manifest_args!(ManifestExpression::EntireWorktop),
                    )
                    .build();

                vec![stake_manifest, register_manifest]
            }
        }
    }
}

fn register_and_stake_new_validator(
    register_and_stake_txn_type: RegisterAndStakeTransactionType,
    pub_key: EcdsaSecp256k1PublicKey,
    account_address: ComponentAddress,
    stake_amount: Decimal,
    test_runner: &mut TestRunner,
) -> ComponentAddress {
    let validator_address = test_runner.new_validator_with_pub_key(pub_key, account_address);

    let manifests = register_and_stake_txn_type.manifests(
        stake_amount,
        account_address,
        validator_address,
        test_runner.faucet_component(),
    );

    for manifest in manifests {
        let receipt = test_runner.execute_manifest(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&pub_key)],
        );
        receipt.expect_commit_success();
    }

    validator_address
}

fn registered_validator_test(
    register_and_stake_txn_type: RegisterAndStakeTransactionType,
    num_initial_validators: usize,
    max_validators: usize,
    initial_stakes: Decimal,
    validator_to_stake_amount: Decimal,
    expect_in_next_epoch: bool,
    expected_num_validators_in_next_epoch: usize,
) {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let (genesis, accounts) = create_custom_genesis(
        initial_epoch,
        rounds_per_epoch,
        num_initial_validators,
        max_validators,
        initial_stakes,
        validator_to_stake_amount,
        1,
    );
    let (pub_key, account_address) = accounts[0];
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let validator_address = register_and_stake_new_validator(
        register_and_stake_txn_type,
        pub_key,
        account_address,
        validator_to_stake_amount,
        &mut test_runner,
    );

    // Act
    let receipt = test_runner.advance_to_round(rounds_per_epoch);

    // Assert
    let result = receipt.expect_commit_success();
    let next_epoch = result.next_epoch().expect("Should have next epoch");
    assert_eq!(
        next_epoch.validator_set.validators_by_stake_desc.len(),
        expected_num_validators_in_next_epoch
    );
    assert_eq!(next_epoch.epoch, initial_epoch + 1);
    assert_eq!(
        next_epoch
            .validator_set
            .validators_by_stake_desc
            .contains_key(&validator_address),
        expect_in_next_epoch
    );
}

#[test]
fn registered_validator_with_stake_does_not_become_part_of_validator_on_epoch_change_if_stake_not_enough(
) {
    for register_and_stake_type in RegisterAndStakeTransactionType::ALL_TYPES {
        registered_validator_test(
            register_and_stake_type,
            10,
            10,
            1000000.into(),
            900000.into(),
            false,
            10,
        );
    }
}

#[test]
fn registered_validator_with_stake_does_become_part_of_validator_on_epoch_change_if_there_are_empty_spots(
) {
    for register_and_stake_type in RegisterAndStakeTransactionType::ALL_TYPES {
        registered_validator_test(
            register_and_stake_type,
            9,
            10,
            1000000.into(),
            900000.into(),
            true,
            10,
        );
    }
}

#[test]
fn registered_validator_with_enough_stake_does_become_part_of_validator_on_epoch_change() {
    for register_and_stake_type in RegisterAndStakeTransactionType::ALL_TYPES {
        registered_validator_test(
            register_and_stake_type,
            10,
            10,
            1000000.into(),
            1100000.into(),
            true,
            10,
        );
    }
}

#[test]
fn low_stakes_should_cause_no_problems() {
    for register_and_stake_type in RegisterAndStakeTransactionType::ALL_TYPES {
        registered_validator_test(register_and_stake_type, 1, 10, 1.into(), 1.into(), true, 2);
    }
}

#[test]
fn test_registering_and_staking_many_validators() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let (genesis, accounts) = create_custom_genesis(
        initial_epoch,
        rounds_per_epoch,
        1,
        10,
        1.into(),
        1.into(),
        10,
    );
    let mut rng = ChaCha8Rng::seed_from_u64(1234);

    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let mut all_manifests = Vec::new();
    for (pub_key, account_address) in accounts {
        let validator_address = test_runner.new_validator_with_pub_key(pub_key, account_address);

        let rand = rng.gen_range(0..RegisterAndStakeTransactionType::ALL_TYPES.len());
        let register_and_stake_type = RegisterAndStakeTransactionType::ALL_TYPES[rand];

        let manifests = register_and_stake_type.manifests(
            1.into(),
            account_address,
            validator_address,
            test_runner.faucet_component(),
        );
        all_manifests.push((pub_key, manifests));
    }

    all_manifests.shuffle(&mut rng);

    for (pub_key, manifests) in all_manifests {
        for manifest in manifests {
            let receipt = test_runner.execute_manifest(
                manifest,
                vec![NonFungibleGlobalId::from_public_key(&pub_key)],
            );
            receipt.expect_commit_success();
        }
    }

    // Act
    let receipt = test_runner.advance_to_round(rounds_per_epoch);

    // Assert
    let result = receipt.expect_commit_success();
    let next_epoch = result.next_epoch().expect("Should have next epoch");
    assert_eq!(next_epoch.validator_set.validators_by_stake_desc.len(), 10);
    assert_eq!(next_epoch.epoch, initial_epoch + 1);
}

#[test]
fn unregistered_validator_gets_removed_on_epoch_change() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let validator_pub_key = EcdsaSecp256k1PrivateKey::from_u64(2u64)
        .unwrap()
        .public_key();
    let validator_account_address =
        ComponentAddress::virtual_account_from_public_key(&validator_pub_key);
    let genesis = CustomGenesis::single_validator_and_staker(
        validator_pub_key,
        Decimal::one(),
        validator_account_address,
        initial_epoch,
        dummy_consensus_manager_configuration().with_rounds_per_epoch(rounds_per_epoch),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let validator_address = test_runner.get_active_validator_with_key(&validator_pub_key);
    let manifest = ManifestBuilder::new()
        .create_proof_from_account(validator_account_address, VALIDATOR_OWNER_BADGE)
        .lock_fee(test_runner.faucet_component(), 10.into())
        .unregister_validator(validator_address)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&validator_pub_key)],
    );
    receipt.expect_commit_success();

    // Act
    let receipt = test_runner.advance_to_round(rounds_per_epoch);

    // Assert
    let result = receipt.expect_commit_success();
    let next_epoch = result.next_epoch().expect("Should have next epoch");
    assert_eq!(next_epoch.epoch, initial_epoch + 1);
    assert!(!next_epoch
        .validator_set
        .validators_by_stake_desc
        .contains_key(&validator_address));
}

#[test]
fn updated_validator_keys_gets_updated_on_epoch_change() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let validator_pub_key = EcdsaSecp256k1PrivateKey::from_u64(2u64)
        .unwrap()
        .public_key();
    let validator_account_address =
        ComponentAddress::virtual_account_from_public_key(&validator_pub_key);
    let genesis = CustomGenesis::single_validator_and_staker(
        validator_pub_key,
        Decimal::one(),
        validator_account_address,
        initial_epoch,
        dummy_consensus_manager_configuration().with_rounds_per_epoch(rounds_per_epoch),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let validator_address = test_runner.get_active_validator_with_key(&validator_pub_key);
    let next_validator_pub_key = EcdsaSecp256k1PrivateKey::from_u64(3u64)
        .unwrap()
        .public_key();
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account(validator_account_address, VALIDATOR_OWNER_BADGE)
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
    let receipt = test_runner.advance_to_round(rounds_per_epoch);

    // Assert
    let result = receipt.expect_commit_success();
    let next_epoch = result.next_epoch().expect("Should have next epoch");
    assert_eq!(next_epoch.epoch, initial_epoch + 1);
    assert_eq!(
        next_epoch
            .validator_set
            .validators_by_stake_desc
            .get(&validator_address)
            .unwrap()
            .key,
        next_validator_pub_key
    );
}

#[test]
fn cannot_claim_unstake_immediately() {
    // Arrange
    let initial_epoch = 5u64;
    let validator_pub_key = EcdsaSecp256k1PrivateKey::from_u64(2u64)
        .unwrap()
        .public_key();
    let account_pub_key = EcdsaSecp256k1PrivateKey::from_u64(1u64)
        .unwrap()
        .public_key();
    let account_with_su = ComponentAddress::virtual_account_from_public_key(&account_pub_key);
    let genesis = CustomGenesis::single_validator_and_staker(
        validator_pub_key,
        Decimal::from(10),
        account_with_su,
        initial_epoch,
        dummy_consensus_manager_configuration(),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let validator_address = test_runner.get_active_validator_with_key(&validator_pub_key);
    let validator_substate = test_runner.get_validator_info(validator_address);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .withdraw_from_account(
            account_with_su,
            validator_substate.stake_unit_resource,
            1.into(),
        )
        .take_all_from_worktop(validator_substate.stake_unit_resource, |builder, bucket| {
            builder.unstake_validator(validator_address, bucket)
        })
        .take_all_from_worktop(validator_substate.unstake_nft, |builder, bucket| {
            builder.claim_xrd(validator_address, bucket)
        })
        .call_method(
            account_with_su,
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
    let initial_epoch = 5u32;
    let num_unstake_epochs = 7u32;
    let validator_pub_key = EcdsaSecp256k1PrivateKey::from_u64(2u64)
        .unwrap()
        .public_key();
    let account_pub_key = EcdsaSecp256k1PrivateKey::from_u64(1u64)
        .unwrap()
        .public_key();
    let account_with_su = ComponentAddress::virtual_account_from_public_key(&account_pub_key);
    let genesis = CustomGenesis::single_validator_and_staker(
        validator_pub_key,
        Decimal::from(10),
        account_with_su,
        initial_epoch as u64,
        dummy_consensus_manager_configuration().with_num_unstake_epochs(num_unstake_epochs as u64),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let validator_address = test_runner.get_active_validator_with_key(&validator_pub_key);
    let validator_substate = test_runner.get_validator_info(validator_address);
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .withdraw_from_account(
            account_with_su,
            validator_substate.stake_unit_resource,
            1.into(),
        )
        .take_all_from_worktop(validator_substate.stake_unit_resource, |builder, bucket| {
            builder.unstake_validator(validator_address, bucket)
        })
        .call_method(
            account_with_su,
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
        .withdraw_from_account(account_with_su, validator_substate.unstake_nft, 1.into())
        .take_all_from_worktop(validator_substate.unstake_nft, |builder, bucket| {
            builder.claim_xrd(validator_address, bucket)
        })
        .call_method(
            account_with_su,
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
fn owner_can_lock_stake_units() {
    // Arrange
    let total_stake_amount = dec!("10.5");
    let stake_units_to_lock_amount = dec!("2.2");
    let validator_key = EcdsaSecp256k1PrivateKey::from_u64(2u64)
        .unwrap()
        .public_key();
    let validator_account = ComponentAddress::virtual_account_from_public_key(&validator_key);
    let genesis = CustomGenesis::single_validator_and_staker(
        validator_key,
        total_stake_amount,
        validator_account,
        5,
        dummy_consensus_manager_configuration(),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let validator_address = test_runner.get_active_validator_with_key(&validator_key);
    let validator_substate = test_runner.get_validator_info(validator_address);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account(validator_account, VALIDATOR_OWNER_BADGE)
        .withdraw_from_account(
            validator_account,
            validator_substate.stake_unit_resource,
            stake_units_to_lock_amount,
        )
        .take_all_from_worktop(validator_substate.stake_unit_resource, |builder, bucket| {
            builder.call_method(
                validator_address,
                VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT,
                manifest_args!(bucket),
            )
        })
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&validator_key)],
    );

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        test_runner.inspect_vault_balance(validator_substate.locked_owner_stake_unit_vault_id.0),
        Some(stake_units_to_lock_amount)
    );
    assert_eq!(
        test_runner.account_balance(validator_account, validator_substate.stake_unit_resource),
        Some(total_stake_amount - stake_units_to_lock_amount)
    )
}

#[test]
fn owner_can_start_unlocking_stake_units() {
    // Arrange
    let initial_epoch = 7;
    let unlock_epochs_delay = 2;
    let total_stake_amount = dec!("10.5");
    let stake_units_to_lock_amount = dec!("2.2");
    let stake_units_to_unlock_amount = dec!("0.1");
    let validator_key = EcdsaSecp256k1PrivateKey::from_u64(2u64)
        .unwrap()
        .public_key();
    let validator_account = ComponentAddress::virtual_account_from_public_key(&validator_key);
    let genesis = CustomGenesis::single_validator_and_staker(
        validator_key,
        total_stake_amount,
        validator_account,
        initial_epoch,
        dummy_consensus_manager_configuration()
            .with_num_owner_stake_units_unlock_epochs(unlock_epochs_delay),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let validator_address = test_runner.get_active_validator_with_key(&validator_key);
    let stake_unit_resource = test_runner
        .get_validator_info(validator_address)
        .stake_unit_resource;

    // Lock
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account(validator_account, VALIDATOR_OWNER_BADGE)
        .withdraw_from_account(
            validator_account,
            stake_unit_resource,
            stake_units_to_lock_amount,
        )
        .take_all_from_worktop(stake_unit_resource, |builder, bucket| {
            builder.call_method(
                validator_address,
                VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT,
                manifest_args!(bucket),
            )
        })
        .build();
    test_runner
        .execute_manifest(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&validator_key)],
        )
        .expect_commit_success();

    // Act (start unlock)
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account(validator_account, VALIDATOR_OWNER_BADGE)
        .call_method(
            validator_address,
            VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT,
            manifest_args!(stake_units_to_unlock_amount),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&validator_key)],
    );

    // Assert
    receipt.expect_commit_success();
    let substate = test_runner.get_validator_info(validator_address);
    assert_eq!(
        test_runner.inspect_vault_balance(substate.locked_owner_stake_unit_vault_id.0),
        Some(stake_units_to_lock_amount - stake_units_to_unlock_amount) // subtracted from the locked vault
    );
    assert_eq!(
        test_runner.inspect_vault_balance(substate.pending_owner_stake_unit_unlock_vault_id.0),
        Some(stake_units_to_unlock_amount) // moved to the pending vault
    );
    assert_eq!(
        substate.pending_owner_stake_unit_withdrawals, // scheduled for unlock in future
        btreemap!(initial_epoch + unlock_epochs_delay => stake_units_to_unlock_amount)
    );
    assert_eq!(
        test_runner.account_balance(validator_account, stake_unit_resource),
        Some(total_stake_amount - stake_units_to_lock_amount) // NOT in the external vault yet
    )
}

#[test]
fn multiple_pending_owner_stake_unit_withdrawals_stack_up() {
    // Arrange
    let initial_epoch = 7;
    let unlock_epochs_delay = 2;
    let total_stake_amount = dec!("10.5");
    let stake_units_to_lock_amount = dec!("2.2");
    let stake_units_to_unlock_amounts = vec![dec!("0.1"), dec!("0.3"), dec!("1.2")];
    let validator_key = EcdsaSecp256k1PrivateKey::from_u64(2u64)
        .unwrap()
        .public_key();
    let validator_account = ComponentAddress::virtual_account_from_public_key(&validator_key);
    let genesis = CustomGenesis::single_validator_and_staker(
        validator_key,
        total_stake_amount,
        validator_account,
        initial_epoch,
        dummy_consensus_manager_configuration()
            .with_num_owner_stake_units_unlock_epochs(unlock_epochs_delay),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let validator_address = test_runner.get_active_validator_with_key(&validator_key);
    let stake_unit_resource = test_runner
        .get_validator_info(validator_address)
        .stake_unit_resource;

    // Lock
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account(validator_account, VALIDATOR_OWNER_BADGE)
        .withdraw_from_account(
            validator_account,
            stake_unit_resource,
            stake_units_to_lock_amount,
        )
        .take_all_from_worktop(stake_unit_resource, |builder, bucket| {
            builder.call_method(
                validator_address,
                VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT,
                manifest_args!(bucket),
            )
        })
        .build();
    test_runner
        .execute_manifest(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&validator_key)],
        )
        .expect_commit_success();

    // Act (start unlock multiple times in a single epoch)
    let stake_units_to_unlock_total_amount = stake_units_to_unlock_amounts.iter().cloned().sum();
    for stake_units_to_unlock_amount in stake_units_to_unlock_amounts {
        let manifest = ManifestBuilder::new()
            .lock_fee(test_runner.faucet_component(), 10.into())
            .create_proof_from_account(validator_account, VALIDATOR_OWNER_BADGE)
            .call_method(
                validator_address,
                VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT,
                manifest_args!(stake_units_to_unlock_amount),
            )
            .build();
        test_runner
            .execute_manifest(
                manifest,
                vec![NonFungibleGlobalId::from_public_key(&validator_key)],
            )
            .expect_commit_success();
    }

    // Assert
    let substate = test_runner.get_validator_info(validator_address);
    assert_eq!(
        test_runner.inspect_vault_balance(substate.locked_owner_stake_unit_vault_id.0),
        Some(stake_units_to_lock_amount - stake_units_to_unlock_total_amount) // subtracted from the locked vault
    );
    assert_eq!(
        test_runner.inspect_vault_balance(substate.pending_owner_stake_unit_unlock_vault_id.0),
        Some(stake_units_to_unlock_total_amount) // moved to the pending vault
    );
    assert_eq!(
        substate.pending_owner_stake_unit_withdrawals, // scheduled for unlock in future
        btreemap!(initial_epoch + unlock_epochs_delay => stake_units_to_unlock_total_amount)
    );
    assert_eq!(
        test_runner.account_balance(validator_account, stake_unit_resource),
        Some(total_stake_amount - stake_units_to_lock_amount) // NOT in the external vault yet
    )
}

#[test]
fn starting_unlock_of_owner_stake_units_moves_already_available_ones_to_separate_field() {
    // Arrange
    let initial_epoch = 7;
    let unlock_epochs_delay = 2;
    let total_stake_amount = dec!("10.5");
    let stake_units_to_lock_amount = dec!("1.0");
    let stake_units_to_unlock_amount = dec!("0.2");
    let stake_units_to_unlock_next_amount = dec!("0.03");
    let total_to_unlock_amount = stake_units_to_unlock_amount + stake_units_to_unlock_next_amount;
    let validator_key = EcdsaSecp256k1PrivateKey::from_u64(2u64)
        .unwrap()
        .public_key();
    let validator_account = ComponentAddress::virtual_account_from_public_key(&validator_key);
    let genesis = CustomGenesis::single_validator_and_staker(
        validator_key,
        total_stake_amount,
        validator_account,
        initial_epoch,
        dummy_consensus_manager_configuration()
            .with_num_owner_stake_units_unlock_epochs(unlock_epochs_delay),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let validator_address = test_runner.get_active_validator_with_key(&validator_key);
    let stake_unit_resource = test_runner
        .get_validator_info(validator_address)
        .stake_unit_resource;

    // Lock
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account(validator_account, VALIDATOR_OWNER_BADGE)
        .withdraw_from_account(
            validator_account,
            stake_unit_resource,
            stake_units_to_lock_amount,
        )
        .take_all_from_worktop(stake_unit_resource, |builder, bucket| {
            builder.call_method(
                validator_address,
                VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT,
                manifest_args!(bucket),
            )
        })
        .build();
    test_runner
        .execute_manifest(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&validator_key)],
        )
        .expect_commit_success();

    // Start unlock
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account(validator_account, VALIDATOR_OWNER_BADGE)
        .call_method(
            validator_address,
            VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT,
            manifest_args!(stake_units_to_unlock_amount),
        )
        .build();
    test_runner
        .execute_manifest(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&validator_key)],
        )
        .expect_commit_success();

    // Act (start unlock again after sufficient delay)
    test_runner.set_current_epoch((initial_epoch + unlock_epochs_delay) as u32);
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account(validator_account, VALIDATOR_OWNER_BADGE)
        .call_method(
            validator_address,
            VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT,
            manifest_args!(stake_units_to_unlock_next_amount),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&validator_key)],
    );

    // Assert
    receipt.expect_commit_success();
    let substate = test_runner.get_validator_info(validator_address);
    assert_eq!(
        test_runner.inspect_vault_balance(substate.locked_owner_stake_unit_vault_id.0),
        Some(stake_units_to_lock_amount - total_to_unlock_amount) // both amounts started unlocking
    );
    assert_eq!(
        test_runner.inspect_vault_balance(substate.pending_owner_stake_unit_unlock_vault_id.0),
        Some(total_to_unlock_amount) // both amounts are still locked (although one is ready to finish unlocking)
    );
    assert_eq!(
        substate.already_unlocked_owner_stake_unit_amount, // the first unlock is moved to here
        stake_units_to_unlock_amount
    );
    assert_eq!(
        substate.pending_owner_stake_unit_withdrawals, // the "next unlock" is scheduled much later
        btreemap!(initial_epoch + 2 * unlock_epochs_delay => stake_units_to_unlock_next_amount)
    );
}

#[test]
fn owner_can_finish_unlocking_stake_units_after_delay() {
    // Arrange
    let initial_epoch = 7;
    let unlock_epochs_delay = 5;
    let total_stake_amount = dec!("10.5");
    let stake_units_to_lock_amount = dec!("2.2");
    let stake_units_to_unlock_amount = dec!("0.1");
    let validator_key = EcdsaSecp256k1PrivateKey::from_u64(2u64)
        .unwrap()
        .public_key();
    let validator_account = ComponentAddress::virtual_account_from_public_key(&validator_key);
    let genesis = CustomGenesis::single_validator_and_staker(
        validator_key,
        total_stake_amount,
        validator_account,
        initial_epoch,
        dummy_consensus_manager_configuration()
            .with_num_owner_stake_units_unlock_epochs(unlock_epochs_delay),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let validator_address = test_runner.get_active_validator_with_key(&validator_key);
    let stake_unit_resource = test_runner
        .get_validator_info(validator_address)
        .stake_unit_resource;

    // Lock
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account(validator_account, VALIDATOR_OWNER_BADGE)
        .withdraw_from_account(
            validator_account,
            stake_unit_resource,
            stake_units_to_lock_amount,
        )
        .take_all_from_worktop(stake_unit_resource, |builder, bucket| {
            builder.call_method(
                validator_address,
                VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT,
                manifest_args!(bucket),
            )
        })
        .build();
    test_runner
        .execute_manifest(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&validator_key)],
        )
        .expect_commit_success();

    // Start unlock
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account(validator_account, VALIDATOR_OWNER_BADGE)
        .call_method(
            validator_address,
            VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT,
            manifest_args!(stake_units_to_unlock_amount),
        )
        .build();
    test_runner
        .execute_manifest(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&validator_key)],
        )
        .expect_commit_success();

    // Act (finish unlock after sufficient delay)
    test_runner.set_current_epoch((initial_epoch + unlock_epochs_delay) as u32);
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account(validator_account, VALIDATOR_OWNER_BADGE)
        .call_method(
            validator_address,
            VALIDATOR_FINISH_UNLOCK_OWNER_STAKE_UNITS_IDENT,
            manifest_args!(),
        )
        .call_method(
            validator_account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&validator_key)],
    );

    // Assert
    receipt.expect_commit_success();
    let substate = test_runner.get_validator_info(validator_address);
    assert_eq!(
        test_runner.inspect_vault_balance(substate.pending_owner_stake_unit_unlock_vault_id.0),
        Some(Decimal::zero()) // subtracted from the pending vault
    );
    assert_eq!(
        substate.pending_owner_stake_unit_withdrawals,
        btreemap!() // removed from the pending tracker
    );
    assert_eq!(
        test_runner.account_balance(validator_account, stake_unit_resource),
        Some(total_stake_amount - stake_units_to_lock_amount + stake_units_to_unlock_amount)
    )
}

#[test]
fn owner_can_not_finish_unlocking_stake_units_before_delay() {
    // Arrange
    let initial_epoch = 7;
    let unlock_epochs_delay = 5;
    let total_stake_amount = dec!("10.5");
    let stake_units_to_lock_amount = dec!("2.2");
    let stake_units_to_unlock_amount = dec!("0.1");
    let validator_key = EcdsaSecp256k1PrivateKey::from_u64(2u64)
        .unwrap()
        .public_key();
    let validator_account = ComponentAddress::virtual_account_from_public_key(&validator_key);
    let genesis = CustomGenesis::single_validator_and_staker(
        validator_key,
        total_stake_amount,
        validator_account,
        initial_epoch,
        dummy_consensus_manager_configuration()
            .with_num_owner_stake_units_unlock_epochs(unlock_epochs_delay),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let validator_address = test_runner.get_active_validator_with_key(&validator_key);
    let stake_unit_resource = test_runner
        .get_validator_info(validator_address)
        .stake_unit_resource;

    // Lock
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account(validator_account, VALIDATOR_OWNER_BADGE)
        .withdraw_from_account(
            validator_account,
            stake_unit_resource,
            stake_units_to_lock_amount,
        )
        .take_all_from_worktop(stake_unit_resource, |builder, bucket| {
            builder.call_method(
                validator_address,
                VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT,
                manifest_args!(bucket),
            )
        })
        .build();
    test_runner
        .execute_manifest(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&validator_key)],
        )
        .expect_commit_success();

    // Start unlock
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account(validator_account, VALIDATOR_OWNER_BADGE)
        .call_method(
            validator_address,
            VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT,
            manifest_args!(stake_units_to_unlock_amount),
        )
        .build();
    test_runner
        .execute_manifest(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&validator_key)],
        )
        .expect_commit_success();

    // Act (finish unlock after insufficient delay)
    test_runner.set_current_epoch((initial_epoch + unlock_epochs_delay) as u32 / 2);
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account(validator_account, VALIDATOR_OWNER_BADGE)
        .call_method(
            validator_address,
            VALIDATOR_FINISH_UNLOCK_OWNER_STAKE_UNITS_IDENT,
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&validator_key)],
    );

    // Assert
    receipt.expect_commit_success(); // it is a success - simply unlocks nothing
    let substate = test_runner.get_validator_info(validator_address);
    assert_eq!(
        test_runner.inspect_vault_balance(substate.pending_owner_stake_unit_unlock_vault_id.0),
        Some(stake_units_to_unlock_amount) // still in the pending vault
    );
    assert_eq!(
        substate.pending_owner_stake_unit_withdrawals, // still scheduled for unlock in future
        btreemap!(initial_epoch + unlock_epochs_delay => stake_units_to_unlock_amount)
    );
    assert_eq!(
        test_runner.account_balance(validator_account, stake_unit_resource),
        Some(total_stake_amount - stake_units_to_lock_amount) // still NOT in the external vault
    )
}

#[test]
fn unstaked_validator_gets_less_stake_on_epoch_change() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let validator_pub_key = EcdsaSecp256k1PrivateKey::from_u64(2u64)
        .unwrap()
        .public_key();
    let account_pub_key = EcdsaSecp256k1PrivateKey::from_u64(1u64)
        .unwrap()
        .public_key();
    let account_with_su = ComponentAddress::virtual_account_from_public_key(&account_pub_key);

    let genesis = CustomGenesis::single_validator_and_staker(
        validator_pub_key,
        Decimal::from(10),
        account_with_su,
        initial_epoch,
        dummy_consensus_manager_configuration().with_rounds_per_epoch(rounds_per_epoch),
    );
    let mut test_runner = TestRunner::builder().with_custom_genesis(genesis).build();
    let validator_address = test_runner.get_active_validator_with_key(&validator_pub_key);
    let validator_substate = test_runner.get_validator_info(validator_address);
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .withdraw_from_account(
            account_with_su,
            validator_substate.stake_unit_resource,
            Decimal::one(),
        )
        .take_all_from_worktop(validator_substate.stake_unit_resource, |builder, bucket| {
            builder.unstake_validator(validator_address, bucket)
        })
        .call_method(
            account_with_su,
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
    let receipt = test_runner.advance_to_round(rounds_per_epoch);

    // Assert
    let result = receipt.expect_commit_success();
    let next_epoch = result.next_epoch().expect("Should have next epoch");
    assert_eq!(next_epoch.epoch, initial_epoch + 1);
    assert_eq!(
        next_epoch
            .validator_set
            .validators_by_stake_desc
            .get(&validator_address)
            .unwrap(),
        &Validator {
            key: validator_pub_key,
            stake: Decimal::from(9),
        }
    );
}

#[test]
fn consensus_manager_create_should_fail_with_supervisor_privilege() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let mut pre_allocated_ids = BTreeSet::new();
    pre_allocated_ids.insert(CONSENSUS_MANAGER.into());
    pre_allocated_ids.insert(VALIDATOR_OWNER_BADGE.into());
    let receipt = test_runner.execute_system_transaction_with_preallocation(
        vec![InstructionV1::CallFunction {
            package_address: CONSENSUS_MANAGER_PACKAGE,
            blueprint_name: CONSENSUS_MANAGER_BLUEPRINT.to_string(),
            function_name: CONSENSUS_MANAGER_CREATE_IDENT.to_string(),
            args: manifest_args!(
                Into::<[u8; NodeId::LENGTH]>::into(VALIDATOR_OWNER_BADGE),
                Into::<[u8; NodeId::LENGTH]>::into(CONSENSUS_MANAGER),
                1u64,
                dummy_consensus_manager_configuration(),
                120000i64
            ),
        }],
        // No validator proofs
        btreeset![],
        pre_allocated_ids,
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError { .. }))
    });
}

#[test]
fn consensus_manager_create_should_succeed_with_system_privilege() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let mut pre_allocated_ids = BTreeSet::new();
    pre_allocated_ids.insert(CONSENSUS_MANAGER.into());
    pre_allocated_ids.insert(VALIDATOR_OWNER_BADGE.into());
    let receipt = test_runner.execute_system_transaction_with_preallocation(
        vec![InstructionV1::CallFunction {
            package_address: CONSENSUS_MANAGER_PACKAGE,
            blueprint_name: CONSENSUS_MANAGER_BLUEPRINT.to_string(),
            function_name: CONSENSUS_MANAGER_CREATE_IDENT.to_string(),
            args: manifest_args!(
                Into::<[u8; NodeId::LENGTH]>::into(VALIDATOR_OWNER_BADGE),
                Into::<[u8; NodeId::LENGTH]>::into(CONSENSUS_MANAGER),
                1u64,
                dummy_consensus_manager_configuration(),
                120000i64
            ),
        }],
        btreeset![AuthAddresses::system_role()],
        pre_allocated_ids,
    );

    // Assert
    receipt.expect_commit_success();
}

fn dummy_consensus_manager_configuration() -> ConsensusManagerInitialConfiguration {
    ConsensusManagerInitialConfiguration {
        max_validators: 10,
        rounds_per_epoch: 5,
        num_unstake_epochs: 1,
        total_emission_xrd_per_epoch: Decimal::one(),
        min_validator_reliability: Decimal::one(),
        num_owner_stake_units_unlock_epochs: 2,
        num_fee_increase_delay_epochs: 1,
    }
}

fn extract_emitter_node_id(event_type_id: &EventTypeIdentifier) -> NodeId {
    match &event_type_id.0 {
        Emitter::Function(node_id, _, _) => node_id,
        Emitter::Method(node_id, _) => node_id,
    }
    .clone()
}

/// Applies an arbitrary rounding of the given decimal, so that simple `==` comparison is meaningful
/// enough for test purposes.
fn precise_enough(exact: Decimal) -> Decimal {
    exact.round(15, RoundingMode::AwayFromZero)
}
