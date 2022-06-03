#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::engine::ResourceFailure;
use radix_engine::engine::RuntimeError;
use scrypto::call_data;
use scrypto::prelude::*;
use transaction::builder::ManifestBuilder;

#[test]
fn dangling_lazy_map_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("lazy_map");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "LazyMapTest",
            call_data!(dangling_lazy_map()),
        )
        .build();
    let signers = vec![];
    let receipt = test_runner.execute_manifest(manifest, signers);

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    assert_eq!(
        runtime_error,
        RuntimeError::ResourceCheckFailure(ResourceFailure::UnclaimedLazyMap)
    );
}

#[test]
fn can_insert_in_child_nodes() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("lazy_map");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "SuperLazyMap", call_data!(new()))
        .build();
    let signers = vec![];
    let receipt = test_runner.execute_manifest(manifest, signers);

    // Assert
    receipt.result.expect("It should work");
}

#[test]
fn create_mutable_lazy_map_into_map_and_referencing_before_storing() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("lazy_map");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "LazyMapTest",
            call_data!(new_lazy_map_into_map_then_get()),
        )
        .build();
    let signers = vec![];
    let receipt = test_runner.execute_manifest(manifest, signers);

    // Assert
    receipt.result.expect("It should work");
}

#[test]
fn cyclic_map_fails_execution() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("lazy_map");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "CyclicMap", call_data!(new()))
        .build();
    let signers = vec![];
    let receipt = test_runner.execute_manifest(manifest, signers);

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    match runtime_error {
        RuntimeError::CyclicLazyMap(_) => {}
        _ => panic!(
            "Should be a cyclic lazy map error but was {}",
            runtime_error
        ),
    }
}

#[test]
fn self_cyclic_map_fails_execution() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("lazy_map");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "CyclicMap", call_data!(new_self_cyclic()))
        .build();
    let signers = vec![];
    let receipt = test_runner.execute_manifest(manifest, signers);

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    match runtime_error {
        RuntimeError::CyclicLazyMap(_) => {}
        _ => panic!(
            "Should be a cyclic lazy map error but was {}",
            runtime_error
        ),
    }
}

#[test]
fn cannot_remove_lazy_maps() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("lazy_map");
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "LazyMapTest",
            call_data!(new_lazy_map_into_vector()),
        )
        .build();
    let signers = vec![];
    let receipt = test_runner.execute_manifest(manifest, signers);
    let component_address = receipt.new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(component_address, call_data!(clear_vector()))
        .build();
    let signers = vec![];
    let receipt = test_runner.execute_manifest(manifest, signers);

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    match runtime_error {
        RuntimeError::LazyMapRemoved(_) => {}
        _ => panic!("Should be lazy map removed error but was {}", runtime_error),
    }
}

#[test]
fn cannot_overwrite_lazy_maps() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("lazy_map");
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "LazyMapTest",
            call_data!(new_lazy_map_into_lazy_map()),
        )
        .build();
    let signers = vec![];
    let receipt = test_runner.execute_manifest(manifest, signers);
    let component_address = receipt.new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(component_address, call_data!(overwrite_lazy_map()))
        .build();
    let signers = vec![];
    let receipt = test_runner.execute_manifest(manifest, signers);

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    match runtime_error {
        RuntimeError::LazyMapRemoved(_) => {}
        _ => panic!("Should be lazy map removed error but was {}", runtime_error),
    }
}

#[test]
fn create_lazy_map_and_get() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("lazy_map");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "LazyMapTest",
            call_data!(new_lazy_map_with_get()),
        )
        .build();
    let signers = vec![];
    let receipt = test_runner.execute_manifest(manifest, signers);

    // Assert
    receipt.result.expect("It should work");
}

#[test]
fn create_lazy_map_and_put() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("lazy_map");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "LazyMapTest",
            call_data!(new_lazy_map_with_put()),
        )
        .build();
    let signers = vec![];
    let receipt = test_runner.execute_manifest(manifest, signers);

    // Assert
    receipt.result.expect("It should work");
}
