use radix_engine::errors::{RuntimeError, SystemModuleError};
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::metadata::MetadataValue;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn cannot_set_package_metadata_with_no_owner() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let code = wat2wasm(include_str!("wasm/basic_package.wat"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            single_function_package_definition("Test", "f"),
            BTreeMap::new(),
            OwnerRole::None,
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let package_address = receipt.expect_commit(true).new_package_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .set_metadata(
            package_address,
            "name".to_string(),
            MetadataValue::String("best package ever!".to_string()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                AuthError::Unauthorized { .. }
            ))
        )
    });
    let value = test_runner.get_metadata(package_address.into(), "name");
    assert_eq!(value, None);
}

#[test]
fn can_set_package_metadata_with_owner() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let code = wat2wasm(include_str!("wasm/basic_package.wat"));
    let (public_key, _, account) = test_runner.new_account(false);
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package(code, single_function_package_definition("Test", "f"))
        .try_deposit_batch_or_abort(account)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let package_address = receipt.expect_commit(true).new_package_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(account, PACKAGE_OWNER_BADGE, dec!("1"))
        .set_metadata(
            package_address,
            "name".to_string(),
            MetadataValue::String("best package ever!".to_string()),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
    let value = test_runner
        .get_metadata(package_address.into(), "name")
        .expect("Should exist");
    assert_eq!(
        value,
        MetadataValue::String("best package ever!".to_string())
    );
}
