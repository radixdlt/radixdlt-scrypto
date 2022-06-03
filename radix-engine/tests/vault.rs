#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::engine::ResourceFailure;
use radix_engine::engine::RuntimeError;
use scrypto::call_data;
use scrypto::prelude::*;
use transaction::builder::ManifestBuilder;

#[test]
fn non_existent_vault_in_component_creation_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("vault");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "NonExistentVault",
            call_data!(create_component_with_non_existent_vault()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    match runtime_error {
        RuntimeError::VaultNotFound(_) => {}
        _ => panic!("Should be vault not found error but was {}", runtime_error),
    }
}

#[test]
fn non_existent_vault_in_committed_component_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("vault");
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "NonExistentVault", call_data!(new()))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(component_address, call_data!(create_non_existent_vault()))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    match runtime_error {
        RuntimeError::VaultNotFound(_) => {}
        _ => panic!("Should be vault not found error but was {}", runtime_error),
    }
}

#[test]
fn non_existent_vault_in_lazy_map_creation_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("vault");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "NonExistentVault",
            call_data!(create_lazy_map_with_non_existent_vault()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    match runtime_error {
        RuntimeError::VaultNotFound(_) => {}
        _ => panic!("Should be vault not found error but was {}", runtime_error),
    }
}

#[test]
fn non_existent_vault_in_committed_lazy_map_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("vault");
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "NonExistentVault", call_data!(new()))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(
            component_address,
            call_data!(create_non_existent_vault_in_lazy_map()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    match runtime_error {
        RuntimeError::VaultNotFound(_) => {}
        _ => panic!("Should be vault not found error but was {}", runtime_error),
    }
}

#[test]
fn dangling_vault_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("vault");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "VaultTest", call_data!(dangling_vault()))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    assert_eq!(
        runtime_error,
        RuntimeError::ResourceCheckFailure(ResourceFailure::Resource(
            receipt.new_resource_addresses[0]
        ))
    );
}

#[test]
fn create_mutable_vault_into_map() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("vault");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "VaultTest",
            call_data!(new_vault_into_map()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn invalid_double_ownership_of_vault() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("vault");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "VaultTest",
            call_data!(invalid_double_ownership_of_vault()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    match runtime_error {
        RuntimeError::VaultNotFound(_) => {}
        _ => panic!("Should be vault not found error"),
    }
}

#[test]
fn create_mutable_vault_into_map_and_referencing_before_storing() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("vault");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "VaultTest",
            call_data!(new_vault_into_map_then_get()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn cannot_overwrite_vault_in_map() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("vault");
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "VaultTest",
            call_data!(new_vault_into_map()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(component_address, call_data!(overwrite_vault_in_map()))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    match runtime_error {
        RuntimeError::VaultRemoved(_) => {}
        _ => panic!("Should be vault not found error"),
    }
}

#[test]
fn create_mutable_vault_into_vector() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("vault");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "VaultTest",
            call_data!(new_vault_into_vector()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn cannot_remove_vaults() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("vault");
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "VaultTest",
            call_data!(new_vault_into_vector()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(component_address, call_data!(clear_vector()))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    match runtime_error {
        RuntimeError::VaultRemoved(_) => {}
        _ => panic!("Should be vault not found error"),
    }
}

#[test]
fn can_push_vault_into_vector() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("vault");
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "VaultTest",
            call_data!(new_vault_into_vector()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(component_address, call_data!(push_vault_into_vector()))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn create_mutable_vault_with_take() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("vault");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "VaultTest",
            call_data!(new_vault_with_take()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn create_mutable_vault_with_take_non_fungible() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("vault");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "VaultTest",
            call_data!(new_vault_with_take_non_fungible()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn create_mutable_vault_with_get_nonfungible_ids() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("vault");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "VaultTest",
            call_data!(new_vault_with_get_non_fungible_ids()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn create_mutable_vault_with_get_amount() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("vault");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "VaultTest",
            call_data!(new_vault_with_get_amount()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn create_mutable_vault_with_get_resource_manager() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("vault");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "VaultTest",
            call_data!(new_vault_with_get_resource_manager()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.result.expect("Should be okay");
}
