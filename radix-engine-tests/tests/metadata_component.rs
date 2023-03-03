use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::data::manifest_args;

#[test]
fn can_globalize_with_component_metadata() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/metadata_component");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "MetadataComponent",
            "new",
            manifest_args!("key".to_string(), "value".to_string()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.new_component_addresses()[0];

    // Assert
    receipt.expect_commit_success();
    let value = test_runner
        .get_metadata(component_address.into(), "key")
        .expect("Should exist");
    assert_eq!(value, "value");
}

#[test]
fn can_set_metadata_after_globalized() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/metadata_component");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "MetadataComponent",
            "new2",
            manifest_args!("key".to_string(), "value".to_string()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.new_component_addresses()[0];

    // Assert
    receipt.expect_commit_success();
    let value = test_runner
        .get_metadata(component_address.into(), "key")
        .expect("Should exist");
    assert_eq!(value, "value");
}

#[test]
fn can_remove_metadata() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/metadata_component");
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "MetadataComponent",
            "new2",
            manifest_args!("key".to_string(), "value".to_string()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "MetadataComponent",
            "remove_metadata",
            manifest_args!(component_address, "key".to_string()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    let value = test_runner.get_metadata(component_address.into(), "key");
    assert_eq!(value, None);
}
