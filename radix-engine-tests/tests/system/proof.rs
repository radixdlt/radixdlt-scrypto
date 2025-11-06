use radix_common::prelude::*;
use radix_engine_interface::types::FromPublicKey;
use scrypto_test::prelude::*;

#[test]
fn test_clone_fungible_proof() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(account, RORK, 1)
        .create_proof_from_auth_zone_of_all(RORK, "proof1")
        .clone_proof("proof1", "proof2")
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_clone_non_fungible_proof() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let nf_resource = ledger.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(
            account,
            NonFungibleGlobalId::new(nf_resource, NonFungibleLocalId::integer(1)),
        )
        .create_proof_from_auth_zone_of_all(nf_resource, "proof1")
        .clone_proof("proof1", "proof2")
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_drop_named_proofs() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let nf_resource = ledger.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, nf_resource, 1)
        .take_all_from_worktop(nf_resource, "bucket1")
        .create_proof_from_bucket_of_all("bucket1", "proof1")
        .drop_named_proofs()
        .deposit(account, "bucket1")
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}
