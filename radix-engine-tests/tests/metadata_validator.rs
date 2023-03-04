use radix_engine::errors::{ModuleError, RuntimeError};
use radix_engine::system::kernel_modules::auth::AuthError;
use radix_engine::system::node_modules::metadata::MetadataValue;
use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;

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
            Address::Component(component_address),
            "name".to_string(),
            "best package ever!".to_string(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![owner_id]);

    // Assert
    receipt.expect_commit_success();
    let value = test_runner
        .get_metadata(component_address.into(), "name")
        .expect("Should exist");
    assert_eq!(value, MetadataValue::String("best package ever!".to_string()));
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
            Address::Component(component_address),
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
