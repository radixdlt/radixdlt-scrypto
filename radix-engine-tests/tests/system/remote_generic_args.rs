use radix_common::prelude::*;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

#[test]
fn test_same_package_remote_generic_arg_for_non_fungible_data() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("remote_generic_args"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NFD",
            "create_non_fungible_resource_with_remote_type",
            manifest_args!(package_address, "NFD", "Type1"),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
}

#[test]
fn test_same_package_remote_generic_arg_for_key_value_store() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("remote_generic_args"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "KVS",
            "create_key_value_store_with_remote_type",
            manifest_args!(package_address, "KVS", "Type2"),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
}

#[test]
fn test_different_package_remote_generic_arg_for_non_fungible_data() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address1 = ledger.publish_package_simple(PackageLoader::get("remote_generic_args"));
    let package_address2 = ledger.publish_package_simple(PackageLoader::get("remote_generic_args"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address1,
            "NFD",
            "create_non_fungible_resource_with_remote_type",
            manifest_args!(package_address2, "NFD", "Type1"),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
}

#[test]
fn test_different_package_remote_generic_arg_for_key_value_store() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address1 = ledger.publish_package_simple(PackageLoader::get("remote_generic_args"));
    let package_address2 = ledger.publish_package_simple(PackageLoader::get("remote_generic_args"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address1,
            "KVS",
            "create_key_value_store_with_remote_type",
            manifest_args!(package_address2, "KVS", "Type2"),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
}

#[test]
fn test_invalid_remote_types_for_non_fungible_data() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address1 = ledger.publish_package_simple(PackageLoader::get("remote_generic_args"));
    let package_address2 = ledger.publish_package_simple(PackageLoader::get("remote_generic_args"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address1,
            "NFD",
            "create_non_fungible_resource_with_remote_type",
            manifest_args!(package_address2, "KVS", "Nonexistent"),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_failure_containing_error("BlueprintTypeNotFound");
}

#[test]
fn test_invalid_remote_types_for_key_value_store() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address1 = ledger.publish_package_simple(PackageLoader::get("remote_generic_args"));
    let package_address2 = ledger.publish_package_simple(PackageLoader::get("remote_generic_args"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address1,
            "NFD",
            "create_key_value_store_with_remote_type",
            manifest_args!(package_address2, "KVS", "Nonexistent"),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_failure_containing_error("BlueprintPayloadDoesNotExist");
}
