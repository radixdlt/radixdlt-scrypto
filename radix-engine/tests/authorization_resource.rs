#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::errors::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use radix_engine::model::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn cannot_mint_with_wrong_auth() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, account) = test_runner.new_public_key_with_account();
    let (_, random_resource_def_id) = test_runner.create_restricted_mint_token(account);
    let (_, token_resource_def_id) = test_runner.create_restricted_mint_token(account);

    // Act
    let fungible_amount = ResourceSpecification::Fungible {
        amount: Decimal::one(),
        resource_def_id: random_resource_def_id,
    };
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(&fungible_amount, account)
        .mint(
            Decimal::from("1.0"),
            token_resource_def_id,
            random_resource_def_id,
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = test_runner.run(transaction);

    // Assert
    let err = receipt.result.expect_err("Should be a runtime error");
    assert_eq!(
        err,
        RuntimeError::ResourceDefError(ResourceDefError::PermissionNotAllowed)
    );
}

#[test]
fn can_mint_with_right_auth() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, account) = test_runner.new_public_key_with_account();
    let (auth_token_resource_def_id, token_resource_def_id) =
        test_runner.create_restricted_mint_token(account);

    // Act
    let fungible_amount = ResourceSpecification::Fungible {
        amount: Decimal::one(),
        resource_def_id: auth_token_resource_def_id,
    };
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(&fungible_amount, account)
        .mint(
            Decimal::from("1.0"),
            token_resource_def_id,
            auth_token_resource_def_id,
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = test_runner.run(transaction);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn cannot_burn_with_no_auth() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, account) = test_runner.new_public_key_with_account();
    let (_, token_resource_def_id) = test_runner.create_restricted_burn_token(account);

    // Act
    let fungible_amount = ResourceSpecification::Fungible {
        amount: Decimal::one(),
        resource_def_id: token_resource_def_id,
    };
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(&fungible_amount, account)
        .burn(&fungible_amount)
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = test_runner.run(transaction);

    // Assert
    let err = receipt.result.expect_err("Should be a runtime error");
    assert_eq!(
        err,
        RuntimeError::ResourceDefError(ResourceDefError::PermissionNotAllowed)
    );
}

#[test]
fn can_burn_with_auth() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, account) = test_runner.new_public_key_with_account();
    let (auth_token_resource_def_id, token_resource_def_id) =
        test_runner.create_restricted_burn_token(account);

    // Act
    let auth_amount = ResourceSpecification::Fungible {
        amount: Decimal::one(),
        resource_def_id: auth_token_resource_def_id,
    };
    let burn_amount = ResourceSpecification::Fungible {
        amount: Decimal::one(),
        resource_def_id: token_resource_def_id,
    };
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(&auth_amount, account)
        .withdraw_from_account(&burn_amount, account)
        .take_from_worktop(&auth_amount, |builder, bucket_id| {
            builder.create_bucket_proof(bucket_id, |builder, proof_id| {
                builder.push_auth(proof_id);
                builder.burn(&burn_amount);
                builder.pop_auth(|builder, proof_id| builder.drop_proof(proof_id))
            })
        })
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = test_runner.run(transaction);

    // Assert
    assert!(receipt.result.is_ok());
}
