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
    let (key, account) = test_runner.new_public_key_with_account();
    let (_, other_account) = test_runner.new_public_key_with_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(RADIX_TOKEN, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = test_runner.run(transaction);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn can_withdraw_non_fungible_from_my_account() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, account) = test_runner.new_public_key_with_account();
    let (_, other_account) = test_runner.new_public_key_with_account();
    let resource_def_id = test_runner.create_non_fungible_resource(account);

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(resource_def_id, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = test_runner.run(transaction);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn cannot_withdraw_from_other_account() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (_, account) = test_runner.new_public_key_with_account();
    let (other_key, other_account) = test_runner.new_public_key_with_account();
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(RADIX_TOKEN, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![other_key])
        .unwrap();

    // Act
    let receipt = test_runner.run(transaction);

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    assert_eq!(runtime_error, RuntimeError::NotAuthorized);
}

#[test]
fn account_to_bucket_to_account() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, account) = test_runner.new_public_key_with_account();
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
        .build(vec![key])
        .unwrap();

    // Act
    let receipt = test_runner.run(transaction);

    // Assert
    assert!(receipt.result.is_ok());
}
