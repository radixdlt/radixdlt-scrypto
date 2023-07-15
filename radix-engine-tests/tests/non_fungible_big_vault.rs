use radix_engine::types::*;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn get_non_fungibles_on_big_vault_with_constraint_should_not_fail() {
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
    // Add 1000 non fungibles to vault
    for _ in 0..100 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(component_address, "mint", manifest_args!(100usize))
            .build();
        let receipt = test_runner.execute_manifest(manifest, vec![]);
        receipt.expect_commit_success();
    }

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component_address, "non_fungibles", manifest_args!(100u32))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}
