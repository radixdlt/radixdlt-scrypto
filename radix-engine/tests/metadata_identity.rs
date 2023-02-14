use radix_engine::engine::{AuthError, ModuleError, RuntimeError};
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::signing::EcdsaSecp256k1PrivateKey;

fn create_identity(
    test_runner: &mut TestRunner,
    pk: EcdsaSecp256k1PublicKey,
    is_virtual: bool,
) -> ComponentAddress {
    if is_virtual {
        ComponentAddress::virtual_identity_from_public_key(&pk)
    } else {
        let owner_id = NonFungibleGlobalId::from_public_key(&pk);
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET_COMPONENT, 10.into())
            .create_identity(rule!(require(owner_id)))
            .build();
        let receipt = test_runner.execute_manifest(manifest, vec![]);
        receipt.expect_commit_success();
        let component_address = receipt
            .expect_commit()
            .entity_changes
            .new_component_addresses[0];

        component_address
    }
}

fn can_set_identity_metadata_with_owner(is_virtual: bool) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let pk = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key();
    let owner_id = NonFungibleGlobalId::from_public_key(&pk);
    let component_address = create_identity(&mut test_runner, pk.clone(), is_virtual);

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
    let component_address = create_identity(&mut test_runner, pk.clone(), is_virtual);

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

#[test]
fn cannot_set_virtual_identity_metadata_without_owner() {
    cannot_set_identity_metadata_without_owner(true);
}

#[test]
fn cannot_set_allocated_identity_metadata_without_owner() {
    cannot_set_identity_metadata_without_owner(false);
}
