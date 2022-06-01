#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::engine::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use radix_engine::model::*;
use radix_engine::wasm::default_wasm_engine;
use scrypto::prelude::*;
use scrypto::to_struct;
use scrypto::values::ScryptoValue;

#[test]
fn can_withdraw_from_my_account() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut test_runner = TestRunner::new(&mut substate_store, &mut wasm_engine);
    let (pk, sk, account) = test_runner.new_account();
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(RADIX_TOKEN, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(test_runner.get_nonce([pk]))
        .sign([&sk]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    receipt.result.expect("It should work");
}

#[test]
fn can_withdraw_non_fungible_from_my_account() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut test_runner = TestRunner::new(&mut substate_store, &mut wasm_engine);
    let (pk, sk, account) = test_runner.new_account();
    let (_, _, other_account) = test_runner.new_account();
    let resource_address = test_runner.create_non_fungible_resource(account);

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(resource_address, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(test_runner.get_nonce([pk]))
        .sign([&sk]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn cannot_withdraw_from_other_account() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut test_runner = TestRunner::new(&mut substate_store, &mut wasm_engine);
    let (_, _, account) = test_runner.new_account();
    let (other_pk, other_sk, other_account) = test_runner.new_account();
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(RADIX_TOKEN, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(test_runner.get_nonce([other_pk]))
        .sign([&other_sk]);

    // Act
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    let error = receipt.result.expect_err("Should be runtime error");
    assert_auth_error!(error);
}

#[test]
fn account_to_bucket_to_account() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut test_runner = TestRunner::new(&mut substate_store, &mut wasm_engine);
    let (pk, sk, account) = test_runner.new_account();
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(RADIX_TOKEN, account)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder
                .add_instruction(Instruction::CallMethod {
                    component_address: account,
                    method_name: "deposit".to_string(),
                    arg: to_struct!(scrypto::resource::Bucket(bucket_id)),
                })
                .0
        })
        .build(test_runner.get_nonce([pk]))
        .sign([&sk]);

    // Act
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    receipt.result.expect("It should work");
}

#[test]
fn test_account_balance() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut test_runner = TestRunner::new(&mut substate_store, &mut wasm_engine);
    let (pk, sk, account) = test_runner.new_account();
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(account, "balance", to_struct!(RADIX_TOKEN))
        .build(test_runner.get_nonce([pk]))
        .sign([&sk]);

    // Act
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    assert_eq!(
        receipt.outputs[0],
        ScryptoValue::from_value(&Decimal::from(1000000))
    );
}
