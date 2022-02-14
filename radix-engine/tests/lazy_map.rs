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
fn dangling_lazy_map_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let package = executor.publish_package(&compile("lazy_map")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .call_function(package, "LazyMapTest", "dangling_lazy_map", vec![], None)
        .build(vec![])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();

    // Assert
    assert!(!receipt.result.is_ok());
}

#[test]
fn can_insert_in_child_nodes() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let package = executor.publish_package(&compile("lazy_map")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .call_function(package, "SuperLazyMap", "new", vec![], None)
        .build(vec![])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn create_mutable_lazy_map_into_map_and_referencing_before_storing() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let package = executor.publish_package(&compile("lazy_map")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "LazyMapTest",
            "new_lazy_map_into_map_then_get",
            vec![],
            None,
        )
        .build(vec![])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn cyclic_map_fails_execution() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let package = executor.publish_package(&compile("lazy_map")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .call_function(package, "CyclicMap", "new", vec![], None)
        .build(vec![])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();

    // Assert
    assert!(!receipt.result.is_ok());
}

#[test]
fn self_cyclic_map_fails_execution() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let package = executor.publish_package(&compile("lazy_map")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .call_function(package, "CyclicMap", "new_self_cyclic", vec![], None)
        .build(vec![])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();

    // Assert
    assert!(!receipt.result.is_ok());
}

#[test]
fn cannot_remove_lazy_maps() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, false);
    let package = sut.publish_package(&compile("lazy_map")).unwrap();
    let transaction = TransactionBuilder::new(&sut)
        .call_function(
            package,
            "LazyMapTest",
            "new_lazy_map_into_vector",
            vec![],
            None,
        )
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
fn cannot_overwrite_lazy_maps() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, false);
    let package = sut.publish_package(&compile("lazy_map")).unwrap();
    let transaction = TransactionBuilder::new(&sut)
        .call_function(
            package,
            "LazyMapTest",
            "new_lazy_map_into_lazy_map",
            vec![],
            None,
        )
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
        .call_method(component_address, "overwrite_lazy_map", vec![], None)
        .build(vec![])
        .unwrap();
    let receipt = sut.run(transaction).unwrap();

    // Assert
    assert!(!receipt.result.is_ok());
}

#[test]
fn create_lazy_map_and_get() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let package = executor.publish_package(&compile("lazy_map")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "LazyMapTest",
            "new_lazy_map_with_get",
            vec![],
            None,
        )
        .build(vec![])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn create_lazy_map_and_put() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let package = executor.publish_package(&compile("lazy_map")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "LazyMapTest",
            "new_lazy_map_with_put",
            vec![],
            None,
        )
        .build(vec![])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();

    // Assert
    assert!(receipt.result.is_ok());
}
