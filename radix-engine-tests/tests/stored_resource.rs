use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn stored_resource_is_invokeable() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package = test_runner.compile_and_publish("./tests/blueprints/stored_resource");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(package, "StoredResource", "create", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest2 = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_method(component, "total_supply", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest2, vec![]);

    // Assert
    receipt.expect_commit_success();
}
