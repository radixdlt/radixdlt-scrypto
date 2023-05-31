use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn can_get_from_scrypto() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("../assets/blueprints/metadata");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(package_address, "MetadataTest", "new", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_method(
            component_address,
            "set_array",
            manifest_args!("key", vec![GlobalAddress::from(RADIX_TOKEN)]),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Assert
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_method(component_address, "get_array", manifest_args!("key"))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let x: Vec<GlobalAddress> = receipt.expect_commit_success().output(1);
    assert_eq!(x, vec![GlobalAddress::from(RADIX_TOKEN)])
}

#[test]
fn can_set_from_scrypto() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("../assets/blueprints/metadata");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(package_address, "MetadataTest", "new", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Assert set
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_method(
            component_address,
            "set_array",
            manifest_args!("key", vec![GlobalAddress::from(RADIX_TOKEN)]),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
}
