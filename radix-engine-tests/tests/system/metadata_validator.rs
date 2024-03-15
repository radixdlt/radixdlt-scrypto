use radix_common::prelude::*;
use radix_engine::errors::{RuntimeError, SystemModuleError};
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine_interface::object_modules::metadata::MetadataValue;
use scrypto_test::prelude::*;

#[test]
fn can_set_validator_metadata_with_owner() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pub_key, _, account) = ledger.new_account(false);
    let validator = ledger.new_validator_with_pub_key(pub_key, account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(
            account,
            VALIDATOR_OWNER_BADGE,
            [NonFungibleLocalId::bytes(validator.as_node_id().0).unwrap()],
        )
        .set_metadata(
            validator,
            "name".to_string(),
            MetadataValue::String("best package ever!".to_string()),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );

    // Assert
    receipt.expect_commit_success();
    let value = ledger
        .get_metadata(validator.into(), "name")
        .expect("Should exist");
    assert_eq!(
        value,
        MetadataValue::String("best package ever!".to_string())
    );
}

#[test]
fn cannot_set_validator_metadata_without_owner() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pub_key, _, account) = ledger.new_account(false);
    let validator = ledger.new_validator_with_pub_key(pub_key, account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .set_metadata(
            validator,
            "name".to_string(),
            MetadataValue::String("best package ever!".to_string()),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );

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
