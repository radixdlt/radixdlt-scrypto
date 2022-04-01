#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::errors::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use radix_engine::model::*;
use scrypto::prelude::*;

#[test]
fn can_withdraw_from_my_account() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, sk, account) = test_runner.new_account();
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(RADIX_TOKEN, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![pk])
        .unwrap()
        .sign(&[sk]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn can_withdraw_non_fungible_from_my_account() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, sk, account) = test_runner.new_account();
    let (_, _, other_account) = test_runner.new_account();
    let resource_def_id = test_runner.create_non_fungible_resource(account);

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(resource_def_id, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![pk])
        .unwrap()
        .sign(&[sk]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn cannot_withdraw_from_other_account() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (_, _, account) = test_runner.new_account();
    let (other_pk, other_sk, other_account) = test_runner.new_account();
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(RADIX_TOKEN, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![other_pk])
        .unwrap()
        .sign(&[other_sk]);

    // Act
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    assert_eq!(runtime_error, RuntimeError::NotAuthorized);
}

#[test]
fn account_to_bucket_to_account() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, sk, account) = test_runner.new_account();
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(RADIX_TOKEN, account)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder
                .add_instruction(Instruction::CallMethod {
                    component_id: account,
                    method: "deposit".to_owned(),
                    args: vec![scrypto_encode(&scrypto::resource::Bucket(bucket_id))],
                })
                .0
        })
        .build(vec![pk])
        .unwrap()
        .sign(&[sk]);

    // Act
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    assert!(receipt.result.is_ok());
}
