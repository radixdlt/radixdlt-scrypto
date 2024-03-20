use radix_common::prelude::*;
use radix_engine::errors::{RuntimeError, SystemModuleError};
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine_interface::object_modules::metadata::MetadataValue;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

#[test]
fn cannot_set_package_metadata_with_no_owner() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let code = wat2wasm(include_local_wasm_str!("basic_package.wat"));
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
    let receipt = ledger.execute_manifest(manifest, vec![]);
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
    let value = ledger.get_metadata(package_address.into(), "name");
    assert_eq!(value, None);
}

#[test]
fn can_set_package_metadata_with_owner() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let code = wat2wasm(include_local_wasm_str!("basic_package.wat"));
    let (public_key, _, account) = ledger.new_account(false);
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package(code, single_function_package_definition("Test", "f"))
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let package_address = receipt.expect_commit(true).new_package_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(
            account,
            PACKAGE_OWNER_BADGE,
            [NonFungibleLocalId::bytes(package_address.as_node_id().0).unwrap()],
        )
        .set_metadata(
            package_address,
            "name".to_string(),
            MetadataValue::String("best package ever!".to_string()),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
    let value = ledger
        .get_metadata(package_address.into(), "name")
        .expect("Should exist");
    assert_eq!(
        value,
        MetadataValue::String("best package ever!".to_string())
    );
}
