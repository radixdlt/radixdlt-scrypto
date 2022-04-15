#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::errors::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use scrypto::prelude::*;

#[test]
fn cannot_mint_with_wrong_auth() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, sk, account) = test_runner.new_account();
    let (_, random_resource_address) = test_runner.create_restricted_mint_token(account);
    let (_, token_resource_address) = test_runner.create_restricted_mint_token(account);

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .create_proof_from_account_by_amount(Decimal::one(), random_resource_address, account)
        .mint(
            Decimal::from("1.0"),
            token_resource_address,
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(test_runner.get_nonce([pk]))
        .sign([&sk]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    let err = receipt.result.expect_err("Should be a runtime error");
    assert_auth_error!(err);
}

#[test]
fn can_mint_with_right_auth() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, sk, account) = test_runner.new_account();
    let (auth_token_resource_address, token_resource_address) =
        test_runner.create_restricted_mint_token(account);

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .create_proof_from_account_by_amount(Decimal::one(), auth_token_resource_address, account)
        .mint(
            Decimal::from("1.0"),
            token_resource_address,
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(test_runner.get_nonce([pk]))
        .sign([&sk]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    receipt.result.expect("Should be okay.");
}

#[test]
fn cannot_burn_with_wrong_auth() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, sk, account) = test_runner.new_account();
    let (_, token_resource_address) = test_runner.create_restricted_burn_token(account);

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account_by_amount(Decimal::from(1), token_resource_address, account)
        .create_proof_from_account_by_amount(Decimal::from(1), token_resource_address, account)
        .burn(
            Decimal::one(),
            token_resource_address,
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(test_runner.get_nonce([pk]))
        .sign([&sk]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    let err = receipt.result.expect_err("Should be a runtime error");
    assert_auth_error!(err);
}

#[test]
fn can_burn_with_auth() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, sk, account) = test_runner.new_account();
    let (auth_token_resource_address, token_resource_address) =
        test_runner.create_restricted_burn_token(account);

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .create_proof_from_account_by_amount(Decimal::one(), auth_token_resource_address, account)
        .withdraw_from_account_by_amount(Decimal::one(), token_resource_address, account)
        .burn(
            Decimal::one(),
            token_resource_address,
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(test_runner.get_nonce([pk]))
        .sign([&sk]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    assert!(receipt.result.is_ok());
}
