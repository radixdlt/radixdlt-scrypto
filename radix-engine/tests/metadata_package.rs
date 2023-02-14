use radix_engine::engine::{AuthError, ModuleError, RuntimeError};
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn cannot_set_package_metadata_with_no_owner() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let code = wat2wasm(include_str!("wasm/basic_package.wat"));
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .publish_package(
            code,
            generate_single_function_abi("Test", "f", Type::Any),
            BTreeMap::new(),
            BTreeMap::new(),
            AccessRules::new(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let package_address = receipt.expect_commit().entity_changes.new_package_addresses[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .set_metadata(
            GlobalAddress::Package(package_address),
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
    let metadata = test_runner.get_metadata(GlobalAddress::Package(package_address));
    assert!(metadata.get("name").is_none());
}

#[test]
fn can_set_package_metadata_with_owner() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let code = wat2wasm(include_str!("wasm/basic_package.wat"));
    let (public_key, _, account) = test_runner.new_account(false);
    let owner_badge_resource = test_runner.create_non_fungible_resource(account);
    let owner_badge_addr =
        NonFungibleGlobalId::new(owner_badge_resource, NonFungibleLocalId::integer(1));
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .publish_package_with_owner(
            code,
            generate_single_function_abi("Test", "f", Type::Any),
            owner_badge_addr,
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let package_address = receipt.expect_commit().entity_changes.new_package_addresses[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .create_proof_from_account(account, owner_badge_resource)
        .set_metadata(
            GlobalAddress::Package(package_address),
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
    let metadata = test_runner.get_metadata(GlobalAddress::Package(package_address));
    assert_eq!(metadata.get("name").unwrap(), "best package ever!");
}

#[test]
fn can_lock_package_metadata_with_owner() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let code = wat2wasm(include_str!("wasm/basic_package.wat"));
    let (public_key, _, account) = test_runner.new_account(false);
    let owner_badge_resource = test_runner.create_non_fungible_resource(account);
    let owner_badge_addr =
        NonFungibleGlobalId::new(owner_badge_resource, NonFungibleLocalId::integer(1));
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .publish_package_with_owner(
            code,
            generate_single_function_abi("Test", "f", Type::Any),
            owner_badge_addr,
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let package_address = receipt.expect_commit().entity_changes.new_package_addresses[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .create_proof_from_account(account, owner_badge_resource)
        .set_method_access_rule(
            GlobalAddress::Package(package_address),
            0,
            AccessRuleKey::Native(NativeFn::Metadata(MetadataFn::Set)),
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
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .create_proof_from_account(account, owner_badge_resource)
        .set_metadata(
            GlobalAddress::Package(package_address),
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
    let metadata = test_runner.get_metadata(GlobalAddress::Package(package_address));
    assert!(metadata.get("name").is_none());
}
