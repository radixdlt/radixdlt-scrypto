use radix_common::prelude::*;
use radix_engine::errors::{RuntimeError, SystemModuleError};
use radix_engine::system::system_modules::costing::CostingError;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

fn get_non_fungibles_on_big_vault(size: u32, expect_success: bool) {
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
    // Add 10000 non fungibles to vault
    for _ in 0..100 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(component_address, "mint", manifest_args!(100usize))
            .build();
        let receipt = ledger.execute_manifest(manifest, vec![]);
        receipt.expect_commit_success();
    }

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component_address, "non_fungibles", manifest_args!(size))
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    if expect_success {
        receipt.expect_commit_success();
    } else {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::SystemModuleError(SystemModuleError::CostingError(
                    CostingError::FeeReserveError(..)
                ))
            )
        });
    }
}

#[test]
fn get_non_fungibles_on_big_vault_with_constraint_should_not_fail() {
    get_non_fungibles_on_big_vault(100, true);
}

#[test]
fn get_non_fungibles_on_big_vault_with_no_constraint_should_fail() {
    get_non_fungibles_on_big_vault(u32::MAX, false);
}
