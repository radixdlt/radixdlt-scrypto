use radix_common::prelude::*;
use radix_engine::errors::{RuntimeError, SystemModuleError};
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine_interface::object_modules::metadata::MetadataValue;
use scrypto_test::prelude::*;

fn can_set_identity_metadata_with_owner(is_virtual: bool) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let pk = Secp256k1PrivateKey::from_u64(1).unwrap().public_key();
    let owner_id = NonFungibleGlobalId::from_public_key(&pk);
    let component_address = ledger.new_identity(pk.clone(), is_virtual);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .set_metadata(
            component_address,
            "name",
            MetadataValue::String("best package ever!".to_string()),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![owner_id]);

    // Assert
    receipt.expect_commit_success();
    let value = ledger
        .get_metadata(component_address.into(), "name")
        .expect("Should exist");
    assert_eq!(
        value,
        MetadataValue::String("best package ever!".to_string())
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
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let pk = Secp256k1PrivateKey::from_u64(1).unwrap().public_key();
    let component_address = ledger.new_identity(pk.clone(), is_virtual);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .set_metadata(
            component_address,
            "name".to_string(),
            MetadataValue::String("best package ever!".to_string()),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                AuthError::Unauthorized { .. }
            ))
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
