use radix_engine::errors::SystemModuleError;
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine::{
    blueprints::transaction_processor::TransactionProcessorError,
    errors::{ApplicationError, RuntimeError},
};
use radix_engine_interface::prelude::*;
use radix_engine_interface::types::FromPublicKey;
use scrypto::prelude::{require, require_amount};
use scrypto_test::prelude::*;

#[test]
fn drop_auth_zone_proofs_should_not_drop_named_proofs() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .create_proof_from_account_of_amount(account, XRD, dec!(5))
        .create_proof_from_auth_zone_of_all(XRD, "proof")
        .drop_auth_zone_proofs()
        .drop_proof("proof") // Proof should continue to work after DROP_AUTH_ZONE_PROOFS
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn drop_all_proofs_should_drop_named_proofs() {
    // NB: We disable validation in the manifest builder and leverage the fact that the test runner
    // does not statically validate the manifest.
    // In production, a transaction like what's created here should be rejected because it
    // refers to undefined proof ids.

    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .create_proof_from_account_of_amount(account, XRD, dec!(5))
        .create_proof_from_auth_zone_of_all(XRD, "proof")
        .with_name_lookup(|builder, lookup| {
            // We capture the proof before the lookup knows that the proof has been cleared,
            // which causes a panic in the lookup and would void the test too early!
            let proof = lookup.proof("proof");
            builder.drop_all_proofs().drop_proof(proof) // Proof should fail after DROP_AUTH_ZONE_PROOFS
        })
        .build_no_validate();
    let receipt = ledger.execute_manifest(manifest, [public_key.signature_proof()]);

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
fn drop_auth_zone_signature_proofs_should_invalid_public_key_proof() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let rule = rule!(require(signature(&public_key)));
    let other_account = ledger.new_account_advanced(OwnerRole::Updatable(rule));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .create_proof_from_account_of_amount(account, XRD, dec!(5))
        .drop_auth_zone_signature_proofs()
        .create_proof_from_account_of_amount(other_account, XRD, dec!(1))
        .build();
    let receipt = ledger.execute_manifest(
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
fn drop_auth_zone_signature_proofs_should_not_invalid_physical_proof() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let rule = rule!(require_amount(dec!(5), XRD));
    let other_account = ledger.new_account_advanced(OwnerRole::Updatable(rule));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .create_proof_from_account_of_amount(account, XRD, dec!(5))
        .drop_auth_zone_signature_proofs()
        .create_proof_from_account_of_amount(other_account, XRD, dec!(1))
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}
