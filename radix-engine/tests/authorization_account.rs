#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::errors::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use scrypto::prelude::*;

#[test]
fn can_withdraw_from_my_1_of_2_account_with_key0_sign() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk0, sk0, auth0) = test_runner.new_key_pair_with_pk_address();
    let (_, _, auth1) = test_runner.new_key_pair_with_pk_address();
    let auth_1_of_2 = any_of!(auth0, auth1);
    let account = test_runner.new_account_with_auth_rule(&auth_1_of_2);
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(RADIX_TOKEN, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build_and_sign(vec![pk0], vec![sk0])
        .unwrap();
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn can_withdraw_from_my_1_of_2_account_with_key1_sign() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (_, _, non_fungible_address0) = test_runner.new_key_pair_with_pk_address();
    let (pk1, sk1, non_fungible_address1) = test_runner.new_key_pair_with_pk_address();
    let auth_1_of_2 = any_of!(non_fungible_address0, non_fungible_address1);
    let account = test_runner.new_account_with_auth_rule(&auth_1_of_2);
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(RADIX_TOKEN, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build_and_sign(vec![pk1], vec![sk1])
        .unwrap();
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn can_withdraw_from_my_2_of_2_account_with_both_signatures() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk0, sk0, non_fungible_address0) = test_runner.new_key_pair_with_pk_address();
    let (pk1, sk1, non_fungible_address1) = test_runner.new_key_pair_with_pk_address();
    let auth_2_of_2 = all_of!(non_fungible_address0, non_fungible_address1);
    let account = test_runner.new_account_with_auth_rule(&auth_2_of_2);
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(RADIX_TOKEN, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build_and_sign(vec![pk0, pk1], vec![sk0, sk1])
        .unwrap();
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn cannot_withdraw_from_my_2_of_2_account_with_single_signature() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (_, _, non_fungible_address0) = test_runner.new_key_pair_with_pk_address();
    let (pk1, sk1, non_fungible_address1) = test_runner.new_key_pair_with_pk_address();
    let auth_2_of_2 = all_of!(non_fungible_address0, non_fungible_address1);
    let account = test_runner.new_account_with_auth_rule(&auth_2_of_2);
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(RADIX_TOKEN, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build_and_sign(vec![pk1], vec![sk1])
        .unwrap();
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    let error = receipt.result.expect_err("Should be an error");
    assert_eq!(error, RuntimeError::NotAuthorized);
}

#[test]
fn can_withdraw_from_my_2_of_3_account_with_2_signatures() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (_, _, non_fungible_address0) = test_runner.new_key_pair_with_pk_address();
    let (pk1, sk1, non_fungible_address1) = test_runner.new_key_pair_with_pk_address();
    let (pk2, sk2, non_fungible_address2) = test_runner.new_key_pair_with_pk_address();
    let auth_2_of_3 = min_n_of!(
        2,
        non_fungible_address0,
        non_fungible_address1,
        non_fungible_address2
    );
    let account = test_runner.new_account_with_auth_rule(&auth_2_of_3);
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(RADIX_TOKEN, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build_and_sign(vec![pk1, pk2], vec![sk1, sk2])
        .unwrap();
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn can_withdraw_from_my_any_xrd_auth_account_with_no_signature() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let xrd_auth = this!(RADIX_TOKEN);
    let account = test_runner.new_account_with_auth_rule(&xrd_auth);
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(SYSTEM_COMPONENT, "free_xrd", vec![])
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.create_proof_from_bucket(bucket_id, |builder, proof_id| {
                builder.push_to_auth_zone(proof_id);
                builder.withdraw_from_account(RADIX_TOKEN, account);
                builder.pop_from_auth_zone(|builder, proof_id| builder.drop_proof(proof_id));
                builder
            });
            builder
        })
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn can_withdraw_from_my_any_xrd_auth_account_with_right_amount_of_proof() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let xrd_auth = min_amount_of!(Decimal(1), RADIX_TOKEN);
    let account = test_runner.new_account_with_auth_rule(&xrd_auth);
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(SYSTEM_COMPONENT, "free_xrd", vec![])
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.create_proof_from_bucket(bucket_id, |builder, proof_id| {
                builder.push_to_auth_zone(proof_id);
                builder.withdraw_from_account(RADIX_TOKEN, account);
                builder.pop_from_auth_zone(|builder, proof_id| builder.drop_proof(proof_id));
                builder
            });
            builder
        })
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn cannot_withdraw_from_my_any_xrd_auth_account_with_less_than_amount_of_proof() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let xrd_auth = min_amount_of!(Decimal::from(1), RADIX_TOKEN);
    let account = test_runner.new_account_with_auth_rule(&xrd_auth);
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(SYSTEM_COMPONENT, "free_xrd", vec![])
        .take_from_worktop_by_amount(Decimal::from("0.9"), RADIX_TOKEN, |builder, bucket_id| {
            builder.create_proof_from_bucket(bucket_id, |builder, proof_id| {
                builder.push_to_auth_zone(proof_id);
                builder.withdraw_from_account(RADIX_TOKEN, account);
                builder.pop_from_auth_zone(|builder, proof_id| builder.drop_proof(proof_id));
                builder
            });
            builder
        })
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    let error = receipt.result.expect_err("Should be an error");
    assert_eq!(error, RuntimeError::NotAuthorized);
}
