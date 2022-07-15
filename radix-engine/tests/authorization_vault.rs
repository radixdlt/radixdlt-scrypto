#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::engine::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use scrypto::core::Network;
use scrypto::prelude::*;
use transaction::builder::ManifestBuilder;

#[test]
fn cannot_withdraw_restricted_transfer_from_my_account_with_no_auth() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let (_, _, other_account) = test_runner.new_account();
    let (_, token_resource_address) = test_runner.create_restricted_transfer_token(account);

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .withdraw_from_account_by_amount(Decimal::one(), token_resource_address, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    let err = receipt.result.expect_err("Should be a runtime error");
    assert_auth_error!(err);
}

#[test]
fn can_withdraw_restricted_transfer_from_my_account_with_auth() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let (_, _, other_account) = test_runner.new_account();
    let (auth_resource_address, token_resource_address) =
        test_runner.create_restricted_transfer_token(account);

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .withdraw_from_account_by_ids(
            &BTreeSet::from([NonFungibleId::from_u32(1)]),
            auth_resource_address,
            account,
        )
        .take_from_worktop_by_ids(
            &BTreeSet::from([NonFungibleId::from_u32(1)]),
            auth_resource_address,
            |builder, bucket_id| {
                builder.create_proof_from_bucket(bucket_id, |builder, proof_id| {
                    builder.push_to_auth_zone(proof_id)
                })
            },
        )
        .withdraw_from_account_by_amount(Decimal::one(), token_resource_address, account)
        .pop_from_auth_zone(|builder, proof_id| builder.drop_proof(proof_id))
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_success();
}
