use radix_engine::types::*;
use scrypto_unit::*;
use transaction::prelude::*;

fn get_non_fungibles_on_vault(vault_size: usize, non_fungibles_size: u32, expected_size: usize) {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package = test_runner.compile_and_publish("./tests/blueprints/non_fungible");
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package, "BigVault", "new", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let result = receipt.expect_commit_success();
    let component_address = result.new_component_addresses()[0];
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component_address, "mint", manifest_args!(vault_size))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
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
    let receipt = test_runner.execute_manifest(manifest, vec![]);

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
