#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::errors::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn can_withdraw_from_my_1_of_2_account_with_key0_sign() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key0, auth0) = test_runner.new_public_key_and_non_fungible_address();
    let (_, auth1) = test_runner.new_public_key_and_non_fungible_address();
    let auth_1_of_2 = any_of!(auth0, auth1);
    let account = test_runner.new_account(&auth_1_of_2);
    let (_, other_account) = test_runner.new_public_key_with_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(
            &ResourceSpecifier::Amount(Decimal(100), RADIX_TOKEN),
            account,
        )
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![key0])
        .unwrap();
    let receipt = test_runner.run(transaction);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn can_withdraw_from_my_1_of_2_account_with_key1_sign() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (_, non_fungible_address0) = test_runner.new_public_key_and_non_fungible_address();
    let (key1, non_fungible_address1) = test_runner.new_public_key_and_non_fungible_address();
    let auth_1_of_2 = any_of!(non_fungible_address0, non_fungible_address1);
    let account = test_runner.new_account(&auth_1_of_2);
    let (_, other_account) = test_runner.new_public_key_with_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(
            &ResourceSpecifier::Amount(Decimal(100), RADIX_TOKEN),
            account,
        )
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![key1])
        .unwrap();
    let receipt = test_runner.run(transaction);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn can_withdraw_from_my_2_of_2_account_with_both_signatures() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key0, non_fungible_address0) = test_runner.new_public_key_and_non_fungible_address();
    let (key1, non_fungible_address1) = test_runner.new_public_key_and_non_fungible_address();
    let auth_2_of_2 = all_of!(non_fungible_address0, non_fungible_address1);
    let account = test_runner.new_account(&auth_2_of_2);
    let (_, other_account) = test_runner.new_public_key_with_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(
            &ResourceSpecifier::Amount(Decimal(100), RADIX_TOKEN),
            account,
        )
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![key0, key1])
        .unwrap();
    let receipt = test_runner.run(transaction);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn cannot_withdraw_from_my_2_of_2_account_with_single_signature() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (_, non_fungible_address0) = test_runner.new_public_key_and_non_fungible_address();
    let (key1, non_fungible_address1) = test_runner.new_public_key_and_non_fungible_address();
    let auth_2_of_2 = all_of!(non_fungible_address0, non_fungible_address1);
    let account = test_runner.new_account(&auth_2_of_2);
    let (_, other_account) = test_runner.new_public_key_with_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(
            &ResourceSpecifier::Amount(Decimal(100), RADIX_TOKEN),
            account,
        )
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![key1])
        .unwrap();
    let receipt = test_runner.run(transaction);

    // Assert
    let error = receipt.result.expect_err("Should be an error");
    assert_eq!(error, RuntimeError::NotAuthorized);
}

#[test]
fn can_withdraw_from_my_2_of_3_account_with_2_signatures() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (_, non_fungible_address0) = test_runner.new_public_key_and_non_fungible_address();
    let (key1, non_fungible_address1) = test_runner.new_public_key_and_non_fungible_address();
    let (key2, non_fungible_address2) = test_runner.new_public_key_and_non_fungible_address();
    let auth_2_of_3 = min_n_of!(
        2,
        non_fungible_address0,
        non_fungible_address1,
        non_fungible_address2
    );
    let account = test_runner.new_account(&auth_2_of_3);
    let (_, other_account) = test_runner.new_public_key_with_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(
            &ResourceSpecifier::Amount(Decimal(100), RADIX_TOKEN),
            account,
        )
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![key1, key2])
        .unwrap();
    let receipt = test_runner.run(transaction);

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn can_withdraw_from_my_any_xrd_auth_account_with_no_signature() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let xrd_auth = this!(RADIX_TOKEN);
    let account = test_runner.new_account(&xrd_auth);
    let (_, other_account) = test_runner.new_public_key_with_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(SYSTEM_COMPONENT, "free_xrd", vec![])
        .take_from_worktop(
            &ResourceSpecifier::Amount(Decimal(1), RADIX_TOKEN),
            |builder, bucket_id| {
                builder.create_bucket_proof(bucket_id, |builder, proof_id| {
                    builder.push_onto_auth_zone(proof_id);
                    builder.withdraw_from_account(
                        &ResourceSpecifier::Amount(Decimal(100), RADIX_TOKEN),
                        account,
                    );
                    builder.pop_from_auth_zone(|builder, proof_id| builder.drop_proof(proof_id));
                    builder
                });
                builder
            },
        )
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![])
        .unwrap();
    let receipt = test_runner.run(transaction);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn can_withdraw_from_my_any_xrd_auth_account_with_right_amount_of_proof() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let xrd_auth = min_amount_of!(Decimal(1), RADIX_TOKEN);
    let account = test_runner.new_account(&xrd_auth);
    let (_, other_account) = test_runner.new_public_key_with_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(SYSTEM_COMPONENT, "free_xrd", vec![])
        .take_from_worktop(
            &ResourceSpecifier::Amount(Decimal(1), RADIX_TOKEN),
            |builder, bucket_id| {
                builder.create_bucket_proof(bucket_id, |builder, proof_id| {
                    builder.push_onto_auth_zone(proof_id);
                    builder.withdraw_from_account(
                        &ResourceSpecifier::Amount(Decimal(100), RADIX_TOKEN),
                        account,
                    );
                    builder.pop_from_auth_zone(|builder, proof_id| builder.drop_proof(proof_id));
                    builder
                });
                builder
            },
        )
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![])
        .unwrap();
    let receipt = test_runner.run(transaction);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn cannot_withdraw_from_my_any_xrd_auth_account_with_less_than_amount_of_proof() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let xrd_auth = min_amount_of!(Decimal::from(1), RADIX_TOKEN);
    let account = test_runner.new_account(&xrd_auth);
    let (_, other_account) = test_runner.new_public_key_with_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(SYSTEM_COMPONENT, "free_xrd", vec![])
        .take_from_worktop(
            &ResourceSpecifier::Amount(Decimal::from("0.9"), RADIX_TOKEN),
            |builder, bucket_id| {
                builder.create_bucket_proof(bucket_id, |builder, proof_id| {
                    builder.push_onto_auth_zone(proof_id);
                    builder.withdraw_from_account(
                        &ResourceSpecifier::Amount(Decimal::from(100), RADIX_TOKEN),
                        account,
                    );
                    builder.pop_from_auth_zone(|builder, proof_id| builder.drop_proof(proof_id));
                    builder
                });
                builder
            },
        )
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![])
        .unwrap();
    let receipt = test_runner.run(transaction);

    // Assert
    let error = receipt.result.expect_err("Should be an error");
    assert_eq!(error, RuntimeError::NotAuthorized);
}
