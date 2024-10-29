use scrypto_test::prelude::*;

#[test]
fn v2_transaction_intent_gets_nullified_and_cannot_be_replayed() {
    // This test is similar to test_transaction_replay_protection, but for v1 transactions

    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Act
    let transaction = ledger
        .v2_transaction_builder()
        .manifest(ManifestBuilder::new_v2().lock_fee_from_faucet().build())
        .notarize(&ledger.default_notary())
        .build();

    let receipt = ledger.execute_notarized_transaction(&transaction);

    let commit_result = receipt.expect_commit_success();
    assert_matches!(
        &commit_result.performed_nullifications[..],
        &[Nullification::Intent { intent_hash, .. }] => {
            assert_eq!(intent_hash, transaction.transaction_hashes.transaction_intent_hash.into())
        }
    );

    let duplicate_receipt = ledger.execute_notarized_transaction(&transaction);

    // Assert
    assert_matches!(
        duplicate_receipt.expect_rejection(),
        &RejectionReason::IntentHashPreviouslyCommitted(IntentHash::Transaction(_))
    );
}

#[test]
fn v2_subintent_only_gets_nullified_on_success() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Act
    let partial = ledger
        .v2_partial_transaction_builder()
        .manifest(
            ManifestBuilder::new_subintent_v2()
                .yield_to_parent(())
                .build(),
        )
        .build();

    let failing_transaction = ledger
        .v2_transaction_builder()
        .add_signed_child("child", partial.clone())
        .manifest_builder(
            |builder| {
                builder
                    .lock_fee_from_faucet()
                    .yield_to_child("child", ())
                    .assert_worktop_contains(XRD, 1)
            }, // Fail
        )
        .notarize(&ledger.default_notary())
        .build();

    let receipt = ledger.execute_notarized_transaction(&failing_transaction);
    let commit_result = receipt.expect_commit_failure();
    assert_matches!(
        &commit_result.performed_nullifications[..],
        &[Nullification::Intent { intent_hash, .. }] => {
            assert_eq!(intent_hash, failing_transaction.transaction_hashes.transaction_intent_hash.into())
        }
    );

    let successful_transaction = ledger
        .v2_transaction_builder()
        .add_signed_child("child", partial.clone())
        .manifest_builder(|builder| builder.lock_fee_from_faucet().yield_to_child("child", ()))
        .notarize(&ledger.default_notary())
        .build();

    let receipt = ledger.execute_notarized_transaction(&successful_transaction);
    let commit_result = receipt.expect_commit_success();
    assert_matches!(
        &commit_result.performed_nullifications[..],
        &[
            Nullification::Intent { intent_hash: intent_hash_1, .. },
            Nullification::Intent { intent_hash: intent_hash_2, .. },
        ] => {
            let actually_nullified = indexset!(intent_hash_1, intent_hash_2);
            let expected_nullified: IndexSet<IntentHash> = indexset!(
                successful_transaction.transaction_hashes.transaction_intent_hash.into(),
                partial.root_subintent_hash.into(),
            );
            assert_eq!(actually_nullified, expected_nullified);
        }
    );

    let another_valid_transaction = ledger
        .v2_transaction_builder()
        .add_signed_child("child", partial.clone())
        .manifest_builder(|builder| builder.lock_fee_from_faucet().yield_to_child("child", ()))
        .notarize(&ledger.default_notary())
        .build();

    let receipt = ledger.execute_notarized_transaction(another_valid_transaction);

    assert_matches!(
        receipt.expect_rejection(),
        &RejectionReason::IntentHashPreviouslyCommitted(IntentHash::Subintent(_))
    );
}
