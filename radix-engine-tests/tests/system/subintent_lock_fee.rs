use scrypto_test::prelude::*;

#[test]
fn should_not_be_able_to_lock_fee_in_a_child_subintent() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let (public_key2, _, account2) = ledger.new_allocated_account();

    // Act
    let mut builder = TestTransaction::new_v2_builder(ledger.next_transaction_nonce());

    let child = builder.add_subintent(
        ManifestBuilder::new_subintent_v2()
            .lock_standard_test_fee(account2)
            .yield_to_parent(())
            .build(),
        [public_key2.signature_proof()],
    );

    let transaction = builder.finish_with_root_intent(
        ManifestBuilder::new_v2()
            .use_child("child", child)
            .lock_standard_test_fee(account)
            .yield_to_child("child", ())
            .build(),
        [public_key.signature_proof()],
    );

    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::CannotLockFeeInChildSubintent(..))
        )
    });
}

#[test]
fn should_be_able_to_lock_contingent_fee_in_a_child_subintent() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let (public_key2, _, account2) = ledger.new_allocated_account();

    // Act
    let mut builder = TestTransaction::new_v2_builder(ledger.next_transaction_nonce());

    let child = builder.add_subintent(
        ManifestBuilder::new_subintent_v2()
            .lock_contingent_fee(account2, dec!(10))
            .yield_to_parent(())
            .build(),
        [public_key2.signature_proof()],
    );

    let transaction = builder.finish_with_root_intent(
        ManifestBuilder::new_v2()
            .use_child("child", child)
            .lock_standard_test_fee(account)
            .yield_to_child("child", ())
            .build(),
        [public_key.signature_proof()],
    );

    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    receipt.expect_commit_success();
}
