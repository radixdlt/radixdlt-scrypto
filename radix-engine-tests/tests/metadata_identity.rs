use radix_engine::errors::{ModuleError, RuntimeError};
use radix_engine::system::kernel_modules::auth::AuthError;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::metadata::{MetadataEntry, MetadataValue};
use radix_engine_interface::blueprints::resource::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;

fn can_set_identity_metadata_with_owner(is_virtual: bool) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let pk = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key();
    let owner_id = NonFungibleGlobalId::from_public_key(&pk);
    let component_address = test_runner.new_identity(pk.clone(), is_virtual);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .set_metadata(
            component_address.into(),
            "name".to_string(),
            MetadataEntry::Value(MetadataValue::String("best package ever!".to_string())),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![owner_id]);

    // Assert
    receipt.expect_commit_success();
    let value = test_runner
        .get_metadata(component_address.into(), "name")
        .expect("Should exist");
    assert_eq!(
        value,
        MetadataEntry::Value(MetadataValue::String("best package ever!".to_string()))
    );
}

#[test]
fn can_set_virtual_identity_metadata_with_owner() {
    can_set_identity_metadata_with_owner(true);
}

#[test]
fn can_set_allocated_identity_metadata_with_owner() {
    can_set_identity_metadata_with_owner(false);
}

fn cannot_set_identity_metadata_without_owner(is_virtual: bool) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let pk = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key();
    let component_address = test_runner.new_identity(pk.clone(), is_virtual);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .set_metadata(
            component_address.into(),
            "name".to_string(),
            MetadataEntry::Value(MetadataValue::String("best package ever!".to_string())),
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

#[test]
fn cannot_set_virtual_identity_metadata_without_owner() {
    cannot_set_identity_metadata_without_owner(true);
}

#[test]
fn cannot_set_allocated_identity_metadata_without_owner() {
    cannot_set_identity_metadata_without_owner(false);
}
