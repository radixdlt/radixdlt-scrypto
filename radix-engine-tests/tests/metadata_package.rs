use radix_engine::errors::{ModuleError, RuntimeError};
use radix_engine::system::kernel_modules::auth::AuthError;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::metadata::{
    MetadataEntry, MetadataValue, METADATA_SET_IDENT,
};
use radix_engine_interface::blueprints::account::ACCOUNT_DEPOSIT_BATCH_IDENT;
use radix_engine_interface::blueprints::resource::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn cannot_set_package_metadata_with_no_owner() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let code = wat2wasm(include_str!("wasm/basic_package.wat"));
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .publish_package_advanced(
            code,
            single_function_package_schema("Test", "f"),
            BTreeMap::new(),
            BTreeMap::new(),
            AccessRulesConfig::new(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let package_address = receipt.expect_commit(true).new_package_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .set_metadata(
            package_address.into(),
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
        .lock_fee(test_runner.faucet_component(), 10.into())
        .publish_package(code, single_function_package_schema("Test", "f"))
        .call_method(
            account,
            ACCOUNT_DEPOSIT_BATCH_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let package_address = receipt.expect_commit(true).new_package_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account(account, PACKAGE_OWNER_TOKEN)
        .set_metadata(
            package_address.into(),
            "name".to_string(),
            MetadataEntry::Value(MetadataValue::String("best package ever!".to_string())),
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
        MetadataEntry::Value(MetadataValue::String("best package ever!".to_string()))
    );
}

#[test]
fn can_lock_package_metadata_with_owner() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let code = wat2wasm(include_str!("wasm/basic_package.wat"));
    let (public_key, _, account) = test_runner.new_account(false);
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .publish_package(code, single_function_package_schema("Test", "f"))
        .call_method(
            account,
            ACCOUNT_DEPOSIT_BATCH_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let package_address = receipt.expect_commit(true).new_package_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account(account, PACKAGE_OWNER_TOKEN)
        .set_method_access_rule(
            package_address.into(),
            MethodKey::new(TypedModuleId::Metadata, METADATA_SET_IDENT),
            AccessRule::DenyAll,
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account(account, PACKAGE_OWNER_TOKEN)
        .set_metadata(
            package_address.into(),
            "name".to_string(),
            MetadataEntry::Value(MetadataValue::String("best package ever!".to_string())),
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
    let value = test_runner.get_metadata(package_address.into(), "name");
    assert_eq!(value, None);
}
