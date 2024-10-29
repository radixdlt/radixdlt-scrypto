use radix_transactions::errors::{SubintentStructureError, TransactionValidationError};
use scrypto_test::prelude::*;

#[test]
fn subintents_support_depth_of_four() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, account_key, account) = ledger.new_allocated_account();

    // Act
    let depth_four_child = ledger
        .v2_partial_transaction_builder()
        .manifest_builder(|builder| builder.yield_to_parent(()))
        .build();

    let depth_three_child = ledger
        .v2_partial_transaction_builder()
        .add_signed_child("depth_four_child", depth_four_child)
        .manifest_builder(|builder| {
            builder
                .yield_to_child("depth_four_child", ())
                .yield_to_parent(())
        })
        .build();

    let depth_two_child = ledger
        .v2_partial_transaction_builder()
        .add_signed_child("depth_three_child", depth_three_child)
        .manifest_builder(|builder| {
            builder
                .yield_to_child("depth_three_child", ())
                .yield_to_parent(())
        })
        .build();

    // Root transaction intent is depth 1
    let transaction = ledger
        .v2_transaction_builder()
        .add_signed_child("depth_two_child", depth_two_child)
        .manifest_builder(|builder| {
            builder
                .lock_standard_test_fee(account)
                .yield_to_child("depth_two_child", ())
        })
        .sign(&account_key)
        .notarize(&ledger.default_notary())
        .build();

    ledger
        .execute_notarized_transaction(transaction)
        .expect_commit_success();
}

#[test]
fn subintents_do_not_support_depth_of_five() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, account_key, account) = ledger.new_allocated_account();

    // Act
    let depth_five_child = ledger
        .v2_partial_transaction_builder()
        .manifest_builder(|builder| builder.yield_to_parent(()))
        .build();

    let depth_four_child = ledger
        .v2_partial_transaction_builder()
        .add_signed_child("depth_five_child", depth_five_child)
        .manifest_builder(|builder| {
            builder
                .yield_to_child("depth_five_child", ())
                .yield_to_parent(())
        })
        .build();

    let depth_three_child = ledger
        .v2_partial_transaction_builder()
        .add_signed_child("depth_four_child", depth_four_child)
        .manifest_builder(|builder| {
            builder
                .yield_to_child("depth_four_child", ())
                .yield_to_parent(())
        })
        .build();

    let depth_two_child = ledger
        .v2_partial_transaction_builder()
        .add_signed_child("depth_three_child", depth_three_child)
        .manifest_builder(|builder| {
            builder
                .yield_to_child("depth_three_child", ())
                .yield_to_parent(())
        })
        .build();

    let transaction = ledger
        .v2_transaction_builder()
        .add_signed_child("depth_two_child", depth_two_child)
        .manifest_builder(|builder| {
            builder
                .lock_standard_test_fee(account)
                .yield_to_child("depth_two_child", ())
        })
        .sign(&account_key)
        .notarize(&ledger.default_notary())
        .build_minimal_no_validate();

    let validation_error = transaction
        .prepare_and_validate(ledger.transaction_validator())
        .unwrap_err();

    assert_matches!(
        validation_error,
        TransactionValidationError::SubintentStructureError(
            _,
            SubintentStructureError::SubintentExceedsMaxDepth,
        ),
    );
}
