#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::engine::RuntimeError;
use scrypto::call_data;
use scrypto::prelude::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_package() {
    let mut test_runner = TestRunner::new(true);
    let package = test_runner.publish_package("component");

    let manifest1 = ManifestBuilder::new()
        .call_function(package, "PackageTest", call_data!(publish()))
        .build();
    let receipt1 = test_runner.execute_manifest(manifest1, vec![]);
    assert!(receipt1.result.is_ok());
}

#[test]
fn test_component() {
    let mut test_runner = TestRunner::new(true);
    let (public_key, _, account) = test_runner.new_account();
    let package = test_runner.publish_package("component");

    // Create component
    let manifest1 = ManifestBuilder::new()
        .call_function(package, "ComponentTest", call_data!(create_component()))
        .build();
    let receipt1 = test_runner.execute_manifest(manifest1, vec![]);
    assert!(receipt1.result.is_ok());

    // Find the component address from receipt
    let component = receipt1.new_component_addresses[0];

    // Call functions & methods
    let manifest2 = ManifestBuilder::new()
        .call_function(
            package,
            "ComponentTest",
            call_data![get_component_info(component)],
        )
        .call_method(component, call_data!(get_component_state()))
        .call_method(component, call_data!(put_component_state()))
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt2 = test_runner.execute_manifest(manifest2, vec![public_key]);
    receipt2.result.expect("Should be okay.");
}

#[test]
fn invalid_blueprint_name_should_cause_error() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("component");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "NonExistentBlueprint",
            call_data![create_component()],
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let error = receipt.result.expect_err("Should be an error.");
    assert_eq!(
        error,
        RuntimeError::BlueprintNotFound(package_address, "NonExistentBlueprint".to_string())
    );
}

#[test]
fn reentrancy_should_not_be_possible() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("component");
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "ReentrantComponent", call_data!(new()))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.result.expect("Should be okay");
    let component_address = receipt.new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(component_address, call_data!(call_self()))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let error = receipt.result.expect_err("Should be an error.");
    assert_eq!(error, RuntimeError::ComponentReentrancy(component_address))
}

#[test]
fn missing_component_address_should_cause_error() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let _ = test_runner.publish_package("component");
    let component_address =
        ComponentAddress::from_str("0200000000000000000000000000000000000000000000deadbeef")
            .unwrap();

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(component_address, call_data!(get_component_state()))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let error = receipt.result.expect_err("Should be an error.");
    assert_eq!(error, RuntimeError::ComponentNotFound(component_address));
}
