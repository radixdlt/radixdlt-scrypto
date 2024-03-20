use radix_common::prelude::*;
use radix_engine::errors::{RuntimeError, SystemError};
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

#[test]
fn add_and_remove_of_non_fungible_should_succeed() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package, "AddAndRemove", "new", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let result = receipt.expect_commit_success();
    let component_address = result.new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component_address, "add_and_remove", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn mint_and_burn_of_non_fungible_should_succeed() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package, "MintAndBurn", "new", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let result = receipt.expect_commit_success();
    let component_address = result.new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component_address, "mint_and_burn", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn mint_and_burn_of_non_fungible_2x_should_fail() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package, "MintAndBurn", "new", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let result = receipt.expect_commit_success();
    let component_address = result.new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component_address, "mint_and_burn_2x", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::KeyValueEntryLocked)
        )
    })
}

#[test]
fn mint_of_previously_minted_burned_non_fungible_should_fail() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package, "MintAndBurn", "new", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let result = receipt.expect_commit_success();
    let component_address = result.new_component_addresses()[0];
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component_address, "mint_and_burn", manifest_args!())
        .build();
    ledger.execute_manifest(manifest, vec![]);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component_address, "mint_and_burn", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::KeyValueEntryLocked)
        )
    })
}
