use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn cannot_withdraw_restricted_transfer_from_my_account_with_no_auth() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let (_, _, other_account) = test_runner.new_allocated_account();
    let (_, token_resource_address) = test_runner.create_restricted_transfer_token(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_and_withdraw(account, 10.into(), token_resource_address, Decimal::one())
        .call_method(
            other_account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(is_auth_error);
}

#[test]
fn can_withdraw_restricted_transfer_from_my_account_with_auth() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let (_, _, other_account) = test_runner.new_allocated_account();
    let (auth_resource_address, token_resource_address) =
        test_runner.create_restricted_transfer_token(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_and_withdraw_non_fungibles(
            account,
            10u32.into(),
            auth_resource_address,
            BTreeSet::from([NonFungibleLocalId::integer(1)]),
        )
        .take_from_worktop_by_ids(
            &BTreeSet::from([NonFungibleLocalId::integer(1)]),
            auth_resource_address,
            |builder, bucket_id| {
                builder.create_proof_from_bucket(&bucket_id, |builder, proof_id| {
                    builder
                        .push_to_auth_zone(proof_id)
                        .withdraw_from_account(account, token_resource_address, Decimal::one())
                        .pop_from_auth_zone(|builder, proof_id| builder.drop_proof(proof_id))
                });
                builder.return_to_worktop(bucket_id)
            },
        )
        .call_method(
            other_account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}
