use radix_engine::errors::SystemModuleError;
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine::{
    blueprints::transaction_processor::TransactionProcessorError,
    errors::{ApplicationError, RuntimeError},
    types::*,
};
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto::prelude::{require, require_amount};
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn clear_auth_zone_should_not_drop_named_proofs() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 500u32.into())
        .create_proof_from_account_of_amount(account, RADIX_TOKEN, dec!(5))
        .create_proof_from_auth_zone(RADIX_TOKEN, |builder, proof_id| {
            builder.clear_auth_zone().drop_proof(proof_id) // Proof should continue to work after CLEAR_AUTH_ZONE
        })
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn drop_all_proofs_should_drop_named_proofs() {
    // NB: we're leveraging the fact that test runner does not statically validate the manifest.
    // In production, a transaction like what's created here should be rejected because it
    // refers to undefined proof ids.

    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 500u32.into())
        .create_proof_from_account_of_amount(account, RADIX_TOKEN, dec!(5))
        .create_proof_from_auth_zone(RADIX_TOKEN, |builder, proof_id| {
            builder.drop_all_proofs().drop_proof(proof_id) // Proof should continue to work after CLEAR_AUTH_ZONE
        })
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                TransactionProcessorError::ProofNotFound(0)
            ))
        )
    })
}

#[test]
fn clear_signature_proofs_should_invalid_public_key_proof() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let rule = rule!(require(NonFungibleGlobalId::from_public_key(&public_key)));
    let other_account = test_runner.new_account_advanced(OwnerRole::Updatable(rule));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 500u32.into())
        .create_proof_from_account_of_amount(account, RADIX_TOKEN, dec!(5))
        .clear_signature_proofs()
        .create_proof_from_account_of_amount(other_account, RADIX_TOKEN, dec!(1))
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    })
}

#[test]
fn clear_signature_proofs_should_not_invalid_physical_proof() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let rule = rule!(require_amount(dec!(5), RADIX_TOKEN));
    let other_account = test_runner.new_account_advanced(OwnerRole::Updatable(rule));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 500u32.into())
        .create_proof_from_account_of_amount(account, RADIX_TOKEN, dec!(5))
        .clear_signature_proofs()
        .create_proof_from_account_of_amount(other_account, RADIX_TOKEN, dec!(1))
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}
