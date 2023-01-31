use radix_engine::engine::{AuthError, ModuleError, RuntimeError};
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::signing::EcdsaSecp256k1PrivateKey;

fn create_validator(
    test_runner: &mut TestRunner,
    pk: EcdsaSecp256k1PublicKey,
    owner_access_rule: AccessRule,
) -> ComponentAddress {
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .create_validator(pk, owner_access_rule)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component_address = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    component_address
}

#[test]
fn can_set_validator_metadata_with_owner() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let pk = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key();
    let owner_id = NonFungibleGlobalId::from_public_key(&pk);
    let component_address = create_validator(
        &mut test_runner,
        pk.clone(),
        rule!(require(owner_id.clone())),
    );

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .set_metadata(
            GlobalAddress::Component(component_address),
            "name".to_string(),
            "best package ever!".to_string(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![owner_id]);

    // Assert
    receipt.expect_commit_success();
    let metadata = test_runner.get_metadata(GlobalAddress::Component(component_address));
    assert_eq!(metadata.get("name").unwrap(), "best package ever!");
}

#[test]
fn cannot_set_validator_metadata_without_owner() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let pk = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key();
    let owner_id = NonFungibleGlobalId::from_public_key(&pk);
    let component_address =
        create_validator(&mut test_runner, pk.clone(), rule!(require(owner_id)));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .set_metadata(
            GlobalAddress::Component(component_address),
            "name".to_string(),
            "best package ever!".to_string(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ModuleError(ModuleError::AuthError(AuthError::Unauthorized { .. }))
        )
    });
}
