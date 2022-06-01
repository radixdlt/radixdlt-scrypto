use radix_engine::engine::RuntimeError;
use radix_engine::ledger::*;
use radix_engine::model::extract_package;
use radix_engine::transaction::*;
use radix_engine::wasm::default_wasm_engine;
use scrypto::prelude::*;
use scrypto::to_struct;

#[test]
fn non_existent_vault_in_component_creation_should_fail() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);
    let package = extract_package(compile_package!(format!("./tests/{}", "vault"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(
            package_address,
            "NonExistentVault",
            "create_component_with_non_existent_vault",
            to_struct!(),
        )
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

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
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);
    let package = extract_package(compile_package!(format!("./tests/{}", "vault"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();
    let transaction = TransactionBuilder::new()
        .call_function(package_address, "NonExistentVault", "new", to_struct!())
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();
    let component_address = receipt.new_component_addresses[0];

    // Act
    let transaction = TransactionBuilder::new()
        .call_method(component_address, "create_non_existent_vault", to_struct!())
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

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
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);
    let package = extract_package(compile_package!(format!("./tests/{}", "vault"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(
            package_address,
            "NonExistentVault",
            "create_lazy_map_with_non_existent_vault",
            to_struct!(),
        )
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

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
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);
    let package = extract_package(compile_package!(format!("./tests/{}", "vault"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();
    let transaction = TransactionBuilder::new()
        .call_function(package_address, "NonExistentVault", "new", to_struct!())
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();
    let component_address = receipt.new_component_addresses[0];

    // Act
    let transaction = TransactionBuilder::new()
        .call_method(
            component_address,
            "create_non_existent_vault_in_lazy_map",
            to_struct!(),
        )
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

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
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);
    let package = extract_package(compile_package!(format!("./tests/{}", "vault"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(package_address, "VaultTest", "dangling_vault", to_struct!())
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    assert_eq!(runtime_error, RuntimeError::ResourceCheckFailure);
}

#[test]
fn create_mutable_vault_into_map() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);
    let package = extract_package(compile_package!(format!("./tests/{}", "vault"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_into_map",
            to_struct!(),
        )
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn invalid_double_ownership_of_vault() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);
    let package = extract_package(compile_package!(format!("./tests/{}", "vault"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(
            package_address,
            "VaultTest",
            "invalid_double_ownership_of_vault",
            to_struct!(),
        )
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

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
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);
    let package = extract_package(compile_package!(format!("./tests/{}", "vault"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_into_map_then_get",
            to_struct!(),
        )
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn cannot_overwrite_vault_in_map() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);
    let package = extract_package(compile_package!(format!("./tests/{}", "vault"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();
    let transaction = TransactionBuilder::new()
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_into_map",
            to_struct!(),
        )
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();
    let component_address = receipt.new_component_addresses[0];

    // Act
    let transaction = TransactionBuilder::new()
        .call_method(component_address, "overwrite_vault_in_map", to_struct!())
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

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
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);
    let package = extract_package(compile_package!(format!("./tests/{}", "vault"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_into_vector",
            to_struct!(),
        )
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn cannot_remove_vaults() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);
    let package = extract_package(compile_package!(format!("./tests/{}", "vault"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();
    let transaction = TransactionBuilder::new()
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_into_vector",
            to_struct!(),
        )
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();
    let component_address = receipt.new_component_addresses[0];

    // Act
    let transaction = TransactionBuilder::new()
        .call_method(component_address, "clear_vector", to_struct!())
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

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
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);
    let package = extract_package(compile_package!(format!("./tests/{}", "vault"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();
    let transaction = TransactionBuilder::new()
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_into_vector",
            to_struct!(),
        )
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();
    let component_address = receipt.new_component_addresses[0];

    // Act
    let transaction = TransactionBuilder::new()
        .call_method(component_address, "push_vault_into_vector", to_struct!())
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn create_mutable_vault_with_take() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);
    let package = extract_package(compile_package!(format!("./tests/{}", "vault"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_with_take",
            to_struct!(),
        )
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn create_mutable_vault_with_take_non_fungible() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);
    let package = extract_package(compile_package!(format!("./tests/{}", "vault"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_with_take_non_fungible",
            to_struct!(),
        )
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn create_mutable_vault_with_get_nonfungible_ids() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);
    let package = extract_package(compile_package!(format!("./tests/{}", "vault"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_with_get_non_fungible_ids",
            to_struct!(),
        )
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn create_mutable_vault_with_get_amount() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);
    let package = extract_package(compile_package!(format!("./tests/{}", "vault"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_with_get_amount",
            to_struct!(),
        )
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn create_mutable_vault_with_get_resource_manager() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);
    let package = extract_package(compile_package!(format!("./tests/{}", "vault"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_with_get_resource_manager",
            to_struct!(),
        )
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    receipt.result.expect("Should be okay");
}
