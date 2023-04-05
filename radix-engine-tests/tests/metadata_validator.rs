use radix_engine::errors::{ModuleError, RuntimeError};
use radix_engine::system::kernel_modules::auth::AuthError;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::metadata::{MetadataEntry, MetadataValue};
use radix_engine_interface::blueprints::resource::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn can_set_validator_metadata_with_owner() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (pub_key, _, account) = test_runner.new_account(false);
    let validator = test_runner.new_validator_with_pub_key(pub_key, account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account(account, VALIDATOR_OWNER_TOKEN)
        .set_metadata(
            validator.into(),
            "name".to_string(),
            MetadataEntry::Value(MetadataValue::String("best package ever!".to_string())),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );

    // Assert
    receipt.expect_commit_success();
    let value = test_runner
        .get_metadata(validator.into(), "name")
        .expect("Should exist");
    assert_eq!(
        value,
        MetadataEntry::Value(MetadataValue::String("best package ever!".to_string()))
    );
}

#[test]
fn cannot_set_validator_metadata_without_owner() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (pub_key, _, account) = test_runner.new_account(false);
    let validator = test_runner.new_validator_with_pub_key(pub_key, account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .set_metadata(
            validator.into(),
            "name".to_string(),
            MetadataEntry::Value(MetadataValue::String("best package ever!".to_string())),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ModuleError(ModuleError::AuthError(AuthError::Unauthorized { .. }))
        )
    });
}
