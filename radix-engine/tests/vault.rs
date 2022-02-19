use std::fs;
use std::process::Command;

use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

pub fn compile(name: &str) -> Vec<u8> {
    Command::new("cargo")
        .current_dir(format!("./tests/{}", name))
        .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
        .status()
        .unwrap();
    fs::read(format!(
        "./tests/{}/target/wasm32-unknown-unknown/release/{}.wasm",
        name,
        name.replace("-", "_")
    ))
    .unwrap()
}

#[test]
fn non_existent_vault_in_component_creation_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, false);
    let package = sut.publish_package(&compile("vault")).unwrap();

    // Act
    let result = TransactionBuilder::new(&sut)
        .call_function(
            package,
            "NonExistentVault",
            "create_component_with_non_existent_vault",
            vec![],
            None,
        )
        .build(vec![]);
    let transaction = result.unwrap();
    let receipt = sut.run(transaction).unwrap();
    let result = &receipt.result;

    // Assert
    assert!(!result.is_ok());
}

#[test]
fn non_existent_vault_in_committed_component_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, false);
    let package = sut.publish_package(&compile("vault")).unwrap();
    let result = TransactionBuilder::new(&sut)
        .call_function(
            package,
            "NonExistentVault",
            "new",
            vec![],
            None,
        )
        .build(vec![]);
    let transaction = result.unwrap();
    let receipt = sut.run(transaction).unwrap();
    let component_address = receipt
        .new_entities
        .into_iter()
        .filter(|a| a.is_component())
        .nth(0)
        .unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_method(component_address, "create_non_existent_vault", vec![], None)
        .build(vec![])
        .unwrap();
    let receipt = sut.run(transaction).unwrap();

    // Assert
    let result = &receipt.result;
    assert!(!result.is_ok());
}

#[test]
fn non_existent_vault_in_lazy_map_creation_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, false);
    let package = sut.publish_package(&compile("vault")).unwrap();

    // Act
    let result = TransactionBuilder::new(&sut)
        .call_function(
            package,
            "NonExistentVault",
            "create_lazy_map_with_non_existent_vault",
            vec![],
            None,
        )
        .build(vec![]);
    let transaction = result.unwrap();
    let receipt = sut.run(transaction).unwrap();
    let result = &receipt.result;

    // Assert
    assert!(!result.is_ok());
}


#[test]
fn dangling_vault_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, false);
    let key = sut.new_public_key();
    let account = sut.new_account(key);
    let package = sut.publish_package(&compile("vault")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_function(
            package,
            "VaultTest",
            "dangling_vault",
            vec![],
            Some(account),
        )
        .build(vec![])
        .unwrap();
    let receipt = sut.run(transaction).unwrap();

    // Assert
    assert!(!receipt.result.is_ok());
}

#[test]
fn create_mutable_vault_into_map() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, false);
    let package = sut.publish_package(&compile("vault")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_function(package, "VaultTest", "new_vault_into_map", vec![], None)
        .build(vec![])
        .unwrap();
    let receipt = sut.run(transaction).unwrap();

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn invalid_double_ownership_of_vault() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, false);
    let package = sut.publish_package(&compile("vault")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_function(
            package,
            "VaultTest",
            "invalid_double_ownership_of_vault",
            vec![],
            None,
        )
        .build(vec![])
        .unwrap();
    let receipt = sut.run(transaction).unwrap();

    // Assert
    assert!(!receipt.result.is_ok());
}

#[test]
fn create_mutable_vault_into_map_and_referencing_before_storing() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, false);
    let package = sut.publish_package(&compile("vault")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_function(
            package,
            "VaultTest",
            "new_vault_into_map_then_get",
            vec![],
            None,
        )
        .build(vec![])
        .unwrap();
    let receipt = sut.run(transaction).unwrap();

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn cannot_overwrite_vault_in_map() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, false);
    let package = sut.publish_package(&compile("vault")).unwrap();
    let transaction = TransactionBuilder::new(&sut)
        .call_function(package, "VaultTest", "new_vault_into_map", vec![], None)
        .build(vec![])
        .unwrap();
    let receipt = sut.run(transaction).unwrap();
    let component_address = receipt
        .new_entities
        .into_iter()
        .filter(|a| a.is_component())
        .nth(0)
        .unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_method(component_address, "overwrite_vault_in_map", vec![], None)
        .build(vec![])
        .unwrap();
    let receipt = sut.run(transaction).unwrap();

    // Assert
    assert!(!receipt.result.is_ok());
}

#[test]
fn create_mutable_vault_into_vector() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, false);
    let package = sut.publish_package(&compile("vault")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_function(package, "VaultTest", "new_vault_into_vector", vec![], None)
        .build(vec![])
        .unwrap();
    let receipt = sut.run(transaction).unwrap();

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn cannot_remove_vaults() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, false);
    let package = sut.publish_package(&compile("vault")).unwrap();
    let transaction = TransactionBuilder::new(&sut)
        .call_function(package, "VaultTest", "new_vault_into_vector", vec![], None)
        .build(vec![])
        .unwrap();
    let receipt = sut.run(transaction).unwrap();
    let component_address = receipt
        .new_entities
        .into_iter()
        .filter(|a| a.is_component())
        .nth(0)
        .unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_method(component_address, "clear_vector", vec![], None)
        .build(vec![])
        .unwrap();
    let receipt = sut.run(transaction).unwrap();

    // Assert
    assert!(!receipt.result.is_ok());
}

#[test]
fn can_push_vault_into_vector() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, false);
    let package = sut.publish_package(&compile("vault")).unwrap();
    let transaction = TransactionBuilder::new(&sut)
        .call_function(package, "VaultTest", "new_vault_into_vector", vec![], None)
        .build(vec![])
        .unwrap();
    let receipt = sut.run(transaction).unwrap();
    let component_address = receipt
        .new_entities
        .into_iter()
        .filter(|a| a.is_component())
        .nth(0)
        .unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_method(component_address, "push_vault_into_vector", vec![], None)
        .build(vec![])
        .unwrap();
    let receipt = sut.run(transaction).unwrap();

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn create_mutable_vault_with_take() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, false);
    let package = sut.publish_package(&compile("vault")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_function(package, "VaultTest", "new_vault_with_take", vec![], None)
        .build(vec![])
        .unwrap();
    let receipt = sut.run(transaction).unwrap();

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn create_mutable_vault_with_take_non_fungible() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, false);
    let package = sut.publish_package(&compile("vault")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_function(
            package,
            "VaultTest",
            "new_vault_with_take_non_fungible",
            vec![],
            None,
        )
        .build(vec![])
        .unwrap();
    let receipt = sut.run(transaction).unwrap();

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn create_mutable_vault_with_get_nonfungible_keys() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, false);
    let package = sut.publish_package(&compile("vault")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_function(
            package,
            "VaultTest",
            "new_vault_with_get_non_fungible_keys",
            vec![],
            None,
        )
        .build(vec![])
        .unwrap();
    let receipt = sut.run(transaction).unwrap();

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn create_mutable_vault_with_get_amount() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, false);
    let package = sut.publish_package(&compile("vault")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_function(
            package,
            "VaultTest",
            "new_vault_with_get_amount",
            vec![],
            None,
        )
        .build(vec![])
        .unwrap();
    let receipt = sut.run(transaction).unwrap();

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn create_mutable_vault_with_get_resource_def() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, false);
    let package = sut.publish_package(&compile("vault")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_function(
            package,
            "VaultTest",
            "new_vault_with_get_resource_def",
            vec![],
            None,
        )
        .build(vec![])
        .unwrap();
    let receipt = sut.run(transaction).unwrap();

    // Assert
    assert!(receipt.result.is_ok());
}
