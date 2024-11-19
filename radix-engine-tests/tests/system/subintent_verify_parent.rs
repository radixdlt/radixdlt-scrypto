use radix_common::constants::XRD;
use radix_common::crypto::HasPublicKeyHash;
use radix_engine::errors::{IntentError, RuntimeError, SystemError};
use radix_engine_interface::macros::dec;
use radix_engine_interface::prelude::{require, require_amount, AccessRule};
use radix_engine_interface::rule;
use radix_transactions::builder::ManifestBuilder;
use radix_transactions::model::TestTransaction;
use scrypto_test::ledger_simulator::LedgerSimulatorBuilder;

#[test]
fn should_not_be_able_to_use_subintent_when_verify_parent_access_rule_not_met() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let mut builder = TestTransaction::new_v2_builder(ledger.next_transaction_nonce());

    let child = builder.add_subintent(
        ManifestBuilder::new_subintent_v2()
            .verify_parent(AccessRule::DenyAll)
            .yield_to_parent(())
            .build(),
        [],
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
            RuntimeError::SystemError(SystemError::IntentError(IntentError::VerifyParentFailed))
        )
    });
}

#[test]
fn should_be_able_to_use_subintent_when_verify_parent_access_rule_is_met() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let mut builder = TestTransaction::new_v2_builder(ledger.next_transaction_nonce());

    let child = builder.add_subintent(
        ManifestBuilder::new_subintent_v2()
            .verify_parent(rule!(require(public_key.signature_proof())))
            .yield_to_parent(())
            .build(),
        [],
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

#[test]
fn should_not_be_able_to_use_subintent_when_verify_parent_access_rule_not_met_two_layers() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let mut builder = TestTransaction::new_v2_builder(ledger.next_transaction_nonce());

    let grandchild = builder.add_subintent(
        ManifestBuilder::new_subintent_v2()
            .verify_parent(rule!(require(public_key.signature_proof())))
            .yield_to_parent(())
            .build(),
        [],
    );

    let child = builder.add_subintent(
        ManifestBuilder::new_subintent_v2()
            .use_child("grandchild", grandchild)
            .yield_to_child("grandchild", ())
            .yield_to_parent(())
            .build(),
        [],
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
            RuntimeError::SystemError(SystemError::IntentError(IntentError::VerifyParentFailed))
        )
    });
}

#[test]
fn should_be_able_to_use_subintent_when_verify_parent_access_rule_is_met_two_layers() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, _account) = ledger.new_allocated_account();

    // Act
    let mut builder = TestTransaction::new_v2_builder(ledger.next_transaction_nonce());

    let grandchild = builder.add_subintent(
        ManifestBuilder::new_subintent_v2()
            .verify_parent(rule!(require(public_key.signature_proof())))
            .yield_to_parent(())
            .build(),
        [],
    );

    let child = builder.add_subintent(
        ManifestBuilder::new_subintent_v2()
            .use_child("grandchild", grandchild)
            .yield_to_child("grandchild", ())
            .yield_to_parent(())
            .build(),
        [public_key.signature_proof()],
    );

    let transaction = builder.finish_with_root_intent(
        ManifestBuilder::new_v2()
            .use_child("child", child)
            .lock_fee_from_faucet()
            .yield_to_child("child", ())
            .build(),
        [],
    );

    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn should_be_able_to_use_subintent_when_verify_parent_access_rule_is_met_on_second_yield() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let mut builder = TestTransaction::new_v2_builder(ledger.next_transaction_nonce());

    let child = builder.add_subintent(
        ManifestBuilder::new_subintent_v2()
            .yield_to_parent(())
            .verify_parent(rule!(require_amount(dec!(10), XRD)))
            .yield_to_parent(())
            .build(),
        [],
    );

    let transaction = builder.finish_with_root_intent(
        ManifestBuilder::new_v2()
            .use_child("child", child)
            .lock_standard_test_fee(account)
            .yield_to_child("child", ())
            .create_proof_from_account_of_amount(account, XRD, dec!(10))
            .yield_to_child("child", ())
            .build(),
        [public_key.signature_proof()],
    );

    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    receipt.expect_commit_success();
}
