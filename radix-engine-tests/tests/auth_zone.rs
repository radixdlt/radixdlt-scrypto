use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_auth_zone_create_proof_of_all_for_fungible() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 500u32.into())
        .create_proof_from_account_of_amount(account, RADIX_TOKEN, 10.into())
        .create_proof_from_auth_zone_of_all(RADIX_TOKEN, |builder, proof_id| {
            builder.drop_proof(proof_id)
        })
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_auth_zone_create_proof_of_all_for_non_fungible() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource_address = test_runner.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 500u32.into())
        .create_proof_from_account_of_amount(account, resource_address, 2.into())
        .create_proof_from_auth_zone_of_all(resource_address, |builder, proof_id| {
            builder.drop_proof(proof_id)
        })
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}
