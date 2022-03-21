#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::errors::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use radix_engine::model::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn cannot_withdraw_restricted_transfer_from_my_account_with_no_auth() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, account) = test_runner.new_public_key_with_account();
    let (_, other_account) = test_runner.new_public_key_with_account();
    let (_, token_resource_def_id) = test_runner.create_restricted_transfer_token(account);

    // Act
    let fungible_amount = ResourceSpecifier::Amount(Decimal::one(), token_resource_def_id);
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(&fungible_amount, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
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
fn can_withdraw_restricted_transfer_from_my_account_with_auth() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, account) = test_runner.new_public_key_with_account();
    let (_, other_account) = test_runner.new_public_key_with_account();
    let (auth_resource_def_id, token_resource_def_id) =
        test_runner.create_restricted_transfer_token(account);

    // Act
    let auth_amount = ResourceSpecifier::Ids(
        BTreeSet::from([NonFungibleId::from(1)]),
        auth_resource_def_id,
    );
    let fungible_amount = ResourceSpecifier::Amount(Decimal::one(), token_resource_def_id);
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(&auth_amount, account)
        .take_from_worktop(&auth_amount, |builder, bucket_id| {
            builder.create_bucket_proof(bucket_id, |builder, proof_id| {
                builder.push_onto_auth_zone(proof_id)
            })
        })
        .withdraw_from_account(&fungible_amount, account)
        .pop_from_auth_zone(|builder, proof_id| builder.drop_proof(proof_id))
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = test_runner.run(transaction);

    // Assert
    assert!(receipt.result.is_ok());
}
