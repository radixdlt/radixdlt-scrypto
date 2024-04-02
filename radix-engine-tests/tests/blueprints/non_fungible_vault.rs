use radix_common::prelude::*;
use radix_engine::blueprints::resource::NonFungibleVaultError;
use radix_engine::errors::{ApplicationError, RuntimeError};
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

fn get_non_fungibles_on_vault(vault_size: usize, non_fungibles_size: u32, expected_size: usize) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package, "BigVault", "new", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let result = receipt.expect_commit_success();
    let component_address = result.new_component_addresses()[0];
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component_address, "mint", manifest_args!(vault_size))
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            "non_fungibles",
            manifest_args!(non_fungibles_size),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    let result = receipt.expect_commit_success();
    let ids: BTreeSet<NonFungibleLocalId> = result.output(1);
    assert_eq!(ids.len(), expected_size);
}

#[test]
fn get_non_fungibles_on_vault_with_size_larger_than_vault_size_should_return() {
    get_non_fungibles_on_vault(100, 101, 100);
}

#[test]
fn get_non_fungibles_on_vault_with_size_less_than_vault_size_should_return() {
    get_non_fungibles_on_vault(100, 99, 99);
}

#[test]
fn withdraw_1_from_empty_non_fungible_vault_should_return_error() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package = ledger.publish_package_simple(PackageLoader::get("vault"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleVault",
            "withdraw_one_from_empty",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::NonFungibleVaultError(
                NonFungibleVaultError::NotEnoughAmount
            ))
        )
    });
}
