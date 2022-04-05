use radix_engine::errors::RuntimeError;
use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

pub fn compile(name: &str) -> Vec<u8> {
    compile_package!(format!("./tests/{}", name), name.replace("-", "_"))
}

#[test]
fn dangling_lazy_map_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let package = executor.publish_package(&compile("lazy_map")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .call_function(package, "LazyMapTest", "dangling_lazy_map", vec![])
        .build(&[])
        .unwrap()
        .sign(&[]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    assert_eq!(runtime_error, RuntimeError::ResourceCheckFailure);
}

#[test]
fn can_insert_in_child_nodes() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let package = executor.publish_package(&compile("lazy_map")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .call_function(package, "SuperLazyMap", "new", vec![])
        .build(&[])
        .unwrap()
        .sign(&[]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn create_mutable_lazy_map_into_map_and_referencing_before_storing() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let package = executor.publish_package(&compile("lazy_map")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "LazyMapTest",
            "new_lazy_map_into_map_then_get",
            vec![],
        )
        .build(&[])
        .unwrap()
        .sign(&[]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn cyclic_map_fails_execution() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let package = executor.publish_package(&compile("lazy_map")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .call_function(package, "CyclicMap", "new", vec![])
        .build(&[])
        .unwrap()
        .sign(&[]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

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
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let package = executor.publish_package(&compile("lazy_map")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .call_function(package, "CyclicMap", "new_self_cyclic", vec![])
        .build(&[])
        .unwrap()
        .sign(&[]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

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
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, true);
    let package = sut.publish_package(&compile("lazy_map")).unwrap();
    let transaction = TransactionBuilder::new(&sut)
        .call_function(package, "LazyMapTest", "new_lazy_map_into_vector", vec![])
        .build(&[])
        .unwrap()
        .sign(&[]);
    let receipt = sut.validate_and_execute(&transaction).unwrap();
    let component_address = receipt.new_component_addresses[0];

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_method(component_address, "clear_vector", vec![])
        .build(&[])
        .unwrap()
        .sign(&[]);
    let receipt = sut.validate_and_execute(&transaction).unwrap();

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
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, true);
    let package = sut.publish_package(&compile("lazy_map")).unwrap();
    let transaction = TransactionBuilder::new(&sut)
        .call_function(package, "LazyMapTest", "new_lazy_map_into_lazy_map", vec![])
        .build(&[])
        .unwrap()
        .sign(&[]);
    let receipt = sut.validate_and_execute(&transaction).unwrap();
    let component_address = receipt.new_component_addresses[0];

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_method(component_address, "overwrite_lazy_map", vec![])
        .build(&[])
        .unwrap()
        .sign(&[]);
    let receipt = sut.validate_and_execute(&transaction).unwrap();

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
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let package = executor.publish_package(&compile("lazy_map")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .call_function(package, "LazyMapTest", "new_lazy_map_with_get", vec![])
        .build(&[])
        .unwrap()
        .sign(&[]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn create_lazy_map_and_put() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let package = executor.publish_package(&compile("lazy_map")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .call_function(package, "LazyMapTest", "new_lazy_map_with_put", vec![])
        .build(&[])
        .unwrap()
        .sign(&[]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    assert!(receipt.result.is_ok());
}
