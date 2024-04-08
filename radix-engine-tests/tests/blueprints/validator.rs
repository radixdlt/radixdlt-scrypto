use core::ops::*;

use radix_common::*;
use radix_common::constants::*;
use radix_common::crypto::*;
use radix_common::data::manifest::*;
use radix_common::data::scrypto::*;
use radix_common::data::scrypto::model::*;
use radix_common::math::*;
use radix_common::prelude::*;
use radix_engine::blueprints::consensus_manager::ValidatorError;
use radix_engine::errors::{ApplicationError, RuntimeError};
use radix_engine::transaction::TransactionReceipt;
use radix_engine_interface::*;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::consensus_manager::{
    VALIDATOR_ACCEPTS_DELEGATED_STAKE_IDENT, VALIDATOR_GET_REDEMPTION_VALUE_IDENT,
    ValidatorAcceptsDelegatedStakeInput,
};
use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use radix_engine_interface::types::FromPublicKey;
use radix_transactions::builder::*;
use radix_transactions::signing::*;
use scrypto_test::ledger_simulator::*;

fn signal_protocol_update_test<F>(as_owner: bool, name_len: usize, result_check: F)
    where
        F: Fn(TransactionReceipt),
{
    // Arrange
    let initial_epoch = Epoch::of(5);
    let pub_key = Secp256k1PrivateKey::from_u64(1u64).unwrap().public_key();
    let validator_account_address = ComponentAddress::virtual_account_from_public_key(&pub_key);
    let genesis = CustomGenesis::single_validator_and_staker(
        pub_key,
        Decimal::one(),
        Decimal::ZERO,
        validator_account_address,
        initial_epoch,
        CustomGenesis::default_consensus_manager_config(),
    );
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_genesis(genesis)
        .build();

    // Act
    let validator_address = ledger.get_active_validator_with_key(&pub_key);
    let mut builder = ManifestBuilder::new().lock_fee_from_faucet();
    if as_owner {
        builder = builder.create_proof_from_account_of_non_fungibles(
            validator_account_address,
            VALIDATOR_OWNER_BADGE,
            [NonFungibleLocalId::bytes(validator_address.as_node_id().0).unwrap()],
        );
    }
    let manifest = builder
        .signal_protocol_update_readiness(validator_address, "a".repeat(name_len).as_str())
        .register_validator(validator_address)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );

    // Assert
    result_check(receipt);
}

#[test]
fn can_signal_protocol_update() {
    signal_protocol_update_test(true, 32, |e| {
        e.expect_commit_success();
    })
}

#[test]
fn cannot_signal_protocol_update_if_not_owner() {
    signal_protocol_update_test(false, 32, |e| e.expect_auth_failure())
}

#[test]
fn cannot_signal_protocol_update_if_wrong_length() {
    signal_protocol_update_test(true, 33, |e| {
        e.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::ApplicationError(ApplicationError::ValidatorError(
                    ValidatorError::InvalidProtocolVersionNameLength { .. }
                ))
            )
        });
    })
}

#[test]
fn check_if_validator_accepts_delegated_stake() {
    // Arrange
    let initial_epoch = Epoch::of(5);
    let genesis = CustomGenesis::default(
        initial_epoch,
        CustomGenesis::default_consensus_manager_config(),
    );
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_genesis(genesis)
        .build();
    let (pub_key, _, account) = ledger.new_account(false);

    let validator_address = ledger.new_validator_with_pub_key(pub_key, account);
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(
            account,
            VALIDATOR_OWNER_BADGE,
            [NonFungibleLocalId::bytes(validator_address.as_node_id().0).unwrap()],
        )
        .register_validator(validator_address)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            validator_address,
            VALIDATOR_ACCEPTS_DELEGATED_STAKE_IDENT,
            ValidatorAcceptsDelegatedStakeInput {},
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    let ret = receipt.expect_commit(true).outcome.expect_success();
    assert_eq!(
        ret[1],
        InstructionOutput::CallReturn(scrypto_encode(&false).unwrap())
    );
}

#[test]
fn calling_get_redemption_value_on_staked_validator_with_max_amount_should_not_crash() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pub_key, _, account) = ledger.new_allocated_account();
    let validator_address = ledger.new_staked_validator_with_pub_key(pub_key, account);

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                validator_address,
                VALIDATOR_GET_REDEMPTION_VALUE_IDENT,
                manifest_args!(Decimal::MAX),
            )
            .build(),
        vec![],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::ValidatorError(
                ValidatorError::InvalidGetRedemptionAmount
            ))
        )
    });
}

#[test]
fn calling_get_redemption_value_on_staked_validator_with_smallest_amount_should_not_crash() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pub_key, _, account) = ledger.new_allocated_account();
    let validator_address = ledger.new_staked_validator_with_pub_key(pub_key, account);

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                validator_address,
                VALIDATOR_GET_REDEMPTION_VALUE_IDENT,
                manifest_args!(Decimal(I192::ONE)),
            )
            .build(),
        vec![],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn calling_get_redemption_value_on_staked_validator_with_min_amount_should_not_crash() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pub_key, _, account) = ledger.new_allocated_account();
    let validator_address = ledger.new_staked_validator_with_pub_key(pub_key, account);

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                validator_address,
                VALIDATOR_GET_REDEMPTION_VALUE_IDENT,
                manifest_args!(Decimal::MIN),
            )
            .build(),
        vec![],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::ValidatorError(
                ValidatorError::InvalidGetRedemptionAmount
            ))
        )
    });
}

#[test]
fn calling_get_redemption_value_on_staked_validator_with_zero_amount_should_not_crash() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pub_key, _, account) = ledger.new_allocated_account();
    let validator_address = ledger.new_staked_validator_with_pub_key(pub_key, account);

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                validator_address,
                VALIDATOR_GET_REDEMPTION_VALUE_IDENT,
                manifest_args!(Decimal::ZERO),
            )
            .build(),
        vec![],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::ValidatorError(
                ValidatorError::InvalidGetRedemptionAmount
            ))
        )
    });
}
