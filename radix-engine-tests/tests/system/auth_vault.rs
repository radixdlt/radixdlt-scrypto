use radix_common::prelude::*;
use radix_engine_interface::types::FromPublicKey;
use scrypto_test::prelude::*;

#[test]
fn cannot_withdraw_restricted_transfer_from_my_account_with_no_auth() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let (_, _, other_account) = ledger.new_allocated_account();
    let (_, token_resource_address) = ledger.create_restricted_transfer_token(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_and_withdraw(account, 500, token_resource_address, 1)
        .try_deposit_entire_worktop_or_abort(other_account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(is_auth_error);
}

#[test]
fn can_withdraw_restricted_transfer_from_my_account_with_auth() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let (_, _, other_account) = ledger.new_allocated_account();
    let (auth_resource_address, token_resource_address) =
        ledger.create_restricted_transfer_token(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_and_withdraw_non_fungibles(
            account,
            500,
            auth_resource_address,
            [NonFungibleLocalId::integer(1)],
        )
        .take_non_fungibles_from_worktop(
            auth_resource_address,
            [NonFungibleLocalId::integer(1)],
            "bucket",
        )
        .create_proof_from_bucket_of_all("bucket", "proof")
        .push_to_auth_zone("proof")
        .withdraw_from_account(account, token_resource_address, 1)
        .pop_from_auth_zone("proof2")
        .drop_proof("proof2")
        .return_to_worktop("bucket")
        .try_deposit_entire_worktop_or_abort(other_account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}
