use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn get_global_address_should_succeed() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/address");
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package_address, "MyComponent", "create", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_method(component, "get_address", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let get_address_component: ComponentAddress = receipt.expect_commit(true).output(1);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(component, get_address_component)
}