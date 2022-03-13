pub mod util;

use crate::util::TestUtil;
use radix_engine::errors::RuntimeError;
use radix_engine::ledger::*;
use radix_engine::model::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn cannot_withdraw_restricted_transfer_from_my_account_with_no_auth() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (key, account) = executor.new_public_key_with_account();
    let (_, other_account) = executor.new_public_key_with_account();
    let (_, token_resource_def_id) =
        TestUtil::create_restricted_transfer_token(&mut executor, account);
    let fungible_amount = ResourceSpecification::Fungible {
        amount: Decimal::one(),
        resource_def_id: token_resource_def_id,
    };

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .withdraw_from_account(&fungible_amount, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let result = executor.run(transaction).unwrap();

    // Assert
    let err = result.result.expect_err("Should be a runtime error");
    assert_eq!(
        err,
        RuntimeError::ResourceDefError(ResourceDefError::PermissionNotAllowed)
    );
}

#[test]
fn can_withdraw_restricted_transfer_from_my_account_with_auth() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (key, account) = executor.new_public_key_with_account();
    let (_, other_account) = executor.new_public_key_with_account();
    let (auth_resource_def_id, token_resource_def_id) =
        TestUtil::create_restricted_transfer_token(&mut executor, account);
    let auth_amount = ResourceSpecification::NonFungible {
        keys: BTreeSet::from([NonFungibleId::from(1)]),
        resource_def_id: auth_resource_def_id,
    };
    let fungible_amount = ResourceSpecification::Fungible {
        amount: Decimal::one(),
        resource_def_id: token_resource_def_id,
    };

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .withdraw_from_account(&auth_amount, account)
        .take_from_worktop(&auth_amount, |builder, bucket_id| {
            builder.create_bucket_proof(bucket_id, |builder, proof_id| builder.push_auth(proof_id))
        })
        .withdraw_from_account(&fungible_amount, account)
        .pop_auth(|builder, proof_id| builder.drop_proof(proof_id))
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let result = executor.run(transaction).unwrap();

    // Assert
    assert!(result.result.is_ok());
}
