use radix_engine::engine::{AuthError, ModuleError, RuntimeError};
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn can_set_package_metadata_with_owner() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let (public_key, _, account) = test_runner.new_account(false);
    let owner_badge_resource = test_runner.create_non_fungible_resource(account);
    let owner_badge_id =
        NonFungibleGlobalId::new(owner_badge_resource, NonFungibleLocalId::Number(1));
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .create_identity(rule!(require(owner_badge_id)))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component_address = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .create_proof_from_account(account, owner_badge_resource)
        .set_metadata(
            GlobalAddress::Component(component_address),
            "name".to_string(),
            "best package ever!".to_string(),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
    let metadata = test_runner.get_metadata(GlobalAddress::Component(component_address));
    assert_eq!(metadata.get("name").unwrap(), "best package ever!");
}

#[test]
fn cannot_set_package_metadata_without_owner() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let (public_key, _, account) = test_runner.new_account(false);
    let owner_badge_resource = test_runner.create_non_fungible_resource(account);
    let owner_badge_id =
        NonFungibleGlobalId::new(owner_badge_resource, NonFungibleLocalId::Number(1));
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .create_identity(rule!(require(owner_badge_id)))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component_address = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .set_metadata(
            GlobalAddress::Component(component_address),
            "name".to_string(),
            "best package ever!".to_string(),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ModuleError(ModuleError::AuthError(AuthError::Unauthorized { .. }))
        )
    });
}
