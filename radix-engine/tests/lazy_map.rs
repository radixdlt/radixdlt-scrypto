use radix_engine::engine::RuntimeError;
use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::call_data;
use scrypto::prelude::*;
use radix_engine::model::extract_package;

#[test]
fn dangling_lazy_map_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let package = extract_package(compile_package!(format!("./tests/{}", "lazy_map"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(
            package_address,
            "LazyMapTest",
            call_data!(dangling_lazy_map()),
        )
        .build(executor.get_nonce([]))
        .sign([]);
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
    let package = extract_package(compile_package!(format!("./tests/{}", "lazy_map"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(package_address, "SuperLazyMap", call_data!(new()))
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    receipt.result.expect("It should work");
}

#[test]
fn create_mutable_lazy_map_into_map_and_referencing_before_storing() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let package = extract_package(compile_package!(format!("./tests/{}", "lazy_map"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(
            package_address,
            "LazyMapTest",
            call_data!(new_lazy_map_into_map_then_get()),
        )
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    receipt.result.expect("It should work");
}

#[test]
fn cyclic_map_fails_execution() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let package = extract_package(compile_package!(format!("./tests/{}", "lazy_map"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(package_address, "CyclicMap", call_data!(new()))
        .build(executor.get_nonce([]))
        .sign([]);
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
    let package = extract_package(compile_package!(format!("./tests/{}", "lazy_map"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(package_address, "CyclicMap", call_data!(new_self_cyclic()))
        .build(executor.get_nonce([]))
        .sign([]);
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
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let package = extract_package(compile_package!(format!("./tests/{}", "lazy_map"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();
    let transaction = TransactionBuilder::new()
        .call_function(
            package_address,
            "LazyMapTest",
            call_data!(new_lazy_map_into_vector()),
        )
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();
    let component_address = receipt.new_component_addresses[0];

    // Act
    let transaction = TransactionBuilder::new()
        .call_method(component_address, call_data!(clear_vector()))
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

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
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let package = extract_package(compile_package!(format!("./tests/{}", "lazy_map"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();
    let transaction = TransactionBuilder::new()
        .call_function(
            package_address,
            "LazyMapTest",
            call_data!(new_lazy_map_into_lazy_map()),
        )
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();
    let component_address = receipt.new_component_addresses[0];

    // Act
    let transaction = TransactionBuilder::new()
        .call_method(component_address, call_data!(overwrite_lazy_map()))
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

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
    let package = extract_package(compile_package!(format!("./tests/{}", "lazy_map"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(
            package_address,
            "LazyMapTest",
            call_data!(new_lazy_map_with_get()),
        )
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    receipt.result.expect("It should work");
}

#[test]
fn create_lazy_map_and_put() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let package = extract_package(compile_package!(format!("./tests/{}", "lazy_map"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(
            package_address,
            "LazyMapTest",
            call_data!(new_lazy_map_with_put()),
        )
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    receipt.result.expect("It should work");
}
