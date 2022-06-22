#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::engine::ResourceFailure;
use radix_engine::engine::RuntimeError;
use scrypto::engine::types::StoredValueId;
use scrypto::prelude::*;
use scrypto::to_struct;
use transaction::builder::ManifestBuilder;

#[test]
fn dangling_key_value_store_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("kv_store");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "KeyValueStoreTest",
            "dangling_key_value_store",
            to_struct!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_err(|e| {
        matches!(
            e,
            RuntimeError::ResourceCheckFailure(ResourceFailure::UnclaimedKeyValueStore)
        )
    });
}

#[test]
fn can_insert_in_child_nodes() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("kv_store");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "SuperKeyValueStore", "new", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn create_mutable_key_value_store_into_map_and_referencing_before_storing() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("kv_store");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "KeyValueStoreTest",
            "new_key_value_store_into_map_then_get",
            to_struct!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn cyclic_map_fails_execution() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("kv_store");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "CyclicMap", "new", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    match runtime_error {
        RuntimeError::CyclicKeyValueStore(_) => {}
        _ => panic!(
            "Should be a cyclic key value store error but was {}",
            runtime_error
        ),
    }
}

#[test]
fn self_cyclic_map_fails_execution() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("kv_store");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "CyclicMap",
            "new_self_cyclic",
            to_struct!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    match runtime_error {
        RuntimeError::CyclicKeyValueStore(_) => {}
        _ => panic!(
            "Should be a cyclic key value store error but was {}",
            runtime_error
        ),
    }
}

#[test]
fn cannot_remove_key_value_stores() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("kv_store");
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "KeyValueStoreTest",
            "new_key_value_store_into_vector",
            to_struct!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(component_address, "clear_vector", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    match runtime_error {
        RuntimeError::StoredValueRemoved(StoredValueId::KeyValueStoreId(_)) => {}
        _ => panic!(
            "Should be key value store removed error but was {}",
            runtime_error
        ),
    }
}

#[test]
fn cannot_overwrite_key_value_stores() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("kv_store");
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "KeyValueStoreTest",
            "new_key_value_store_into_key_value_store",
            to_struct!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(component_address, "overwrite_key_value_store", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    match runtime_error {
        RuntimeError::StoredValueRemoved(StoredValueId::KeyValueStoreId(_)) => {}
        _ => panic!(
            "Should be key value store removed error but was {}",
            runtime_error
        ),
    }
}

#[test]
fn create_key_value_store_and_get() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("kv_store");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "KeyValueStoreTest",
            "new_key_value_store_with_get",
            to_struct!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn create_key_value_store_and_put() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("kv_store");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "KeyValueStoreTest",
            "new_key_value_store_with_put",
            to_struct!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn can_reference_in_memory_vault() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("kv_store");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "Precommitted",
            "can_reference_precommitted_vault",
            to_struct!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn can_reference_deep_in_memory_value() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("kv_store");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "Precommitted",
            "can_reference_deep_precommitted_value",
            to_struct!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn can_reference_deep_in_memory_vault() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("kv_store");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "Precommitted",
            "can_reference_deep_precommitted_vault",
            to_struct!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn cannot_directly_reference_inserted_vault() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("kv_store");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "RefCheck",
            "cannot_directly_reference_inserted_vault",
            to_struct!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_err(|e| matches!(e, RuntimeError::ValueNotFound(StoredValueId::VaultId(_))));
}

#[test]
fn cannot_directly_reference_vault_after_container_moved() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("kv_store");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "RefCheck",
            "cannot_directly_reference_vault_after_container_moved",
            to_struct!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_err(|e| matches!(e, RuntimeError::ValueNotFound(StoredValueId::VaultId(_))));
}

#[test]
fn cannot_directly_reference_vault_after_container_stored() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("kv_store");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "RefCheck",
            "cannot_directly_reference_vault_after_container_stored",
            to_struct!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_err(|e| matches!(e, RuntimeError::ValueNotFound(StoredValueId::VaultId(_))));
}
