use radix_engine::errors::RuntimeError;
use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

pub fn compile(name: &str) -> Vec<u8> {
    compile_package!(format!("./tests/{}", name), name.replace("-", "_"))
}

#[test]
fn non_existent_vault_in_component_creation_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, false);
    let package = sut.publish_package(&compile("vault")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_function(
            package,
            "NonExistentVault",
            "create_component_with_non_existent_vault",
            vec![],
        )
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = sut.validate_and_execute(&transaction).unwrap();

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    match runtime_error {
        RuntimeError::VaultNotFound(_) => {}
        _ => panic!("Should be vault not found error"),
    }
}

#[test]
fn non_existent_vault_in_committed_component_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, false);
    let package = sut.publish_package(&compile("vault")).unwrap();
    let transaction = TransactionBuilder::new(&sut)
        .call_function(package, "NonExistentVault", "new", vec![])
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = sut.validate_and_execute(&transaction).unwrap();
    let component_address = receipt.new_component_addresses[0];

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_method(component_address, "create_non_existent_vault", vec![])
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = sut.validate_and_execute(&transaction).unwrap();

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    match runtime_error {
        RuntimeError::VaultNotFound(_) => {}
        _ => panic!("Should be vault not found error"),
    }
}

#[test]
fn non_existent_vault_in_lazy_map_creation_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, false);
    let package = sut.publish_package(&compile("vault")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_function(
            package,
            "NonExistentVault",
            "create_lazy_map_with_non_existent_vault",
            vec![],
        )
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = sut.validate_and_execute(&transaction).unwrap();

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    match runtime_error {
        RuntimeError::VaultNotFound(_) => {}
        _ => panic!("Should be vault not found error"),
    }
}

#[test]
fn non_existent_vault_in_committed_lazy_map_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, false);
    let package = sut.publish_package(&compile("vault")).unwrap();
    let transaction = TransactionBuilder::new(&sut)
        .call_function(package, "NonExistentVault", "new", vec![])
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = sut.validate_and_execute(&transaction).unwrap();
    let component_address = receipt.new_component_addresses[0];

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_method(
            component_address,
            "create_non_existent_vault_in_lazy_map",
            vec![],
        )
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = sut.validate_and_execute(&transaction).unwrap();

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    match runtime_error {
        RuntimeError::VaultNotFound(_) => {}
        _ => panic!("Should be vault not found error"),
    }
}

#[test]
fn dangling_vault_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, true);
    let package = sut.publish_package(&compile("vault")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_function(package, "VaultTest", "dangling_vault", vec![])
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = sut.validate_and_execute(&transaction).unwrap();

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    assert_eq!(runtime_error, RuntimeError::ResourceCheckFailure);
}

#[test]
fn create_mutable_vault_into_map() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, true);
    let package = sut.publish_package(&compile("vault")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_function(package, "VaultTest", "new_vault_into_map", vec![])
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = sut.validate_and_execute(&transaction).unwrap();

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn invalid_double_ownership_of_vault() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, true);
    let package = sut.publish_package(&compile("vault")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_function(
            package,
            "VaultTest",
            "invalid_double_ownership_of_vault",
            vec![],
        )
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = sut.validate_and_execute(&transaction).unwrap();

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
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, true);
    let package = sut.publish_package(&compile("vault")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_function(package, "VaultTest", "new_vault_into_map_then_get", vec![])
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = sut.validate_and_execute(&transaction).unwrap();

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn cannot_overwrite_vault_in_map() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, true);
    let package = sut.publish_package(&compile("vault")).unwrap();
    let transaction = TransactionBuilder::new(&sut)
        .call_function(package, "VaultTest", "new_vault_into_map", vec![])
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = sut.validate_and_execute(&transaction).unwrap();
    let component_address = receipt.new_component_addresses[0];

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_method(component_address, "overwrite_vault_in_map", vec![])
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = sut.validate_and_execute(&transaction).unwrap();

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
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, true);
    let package = sut.publish_package(&compile("vault")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_function(package, "VaultTest", "new_vault_into_vector", vec![])
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = sut.validate_and_execute(&transaction).unwrap();

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn cannot_remove_vaults() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, true);
    let package = sut.publish_package(&compile("vault")).unwrap();
    let transaction = TransactionBuilder::new(&sut)
        .call_function(package, "VaultTest", "new_vault_into_vector", vec![])
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = sut.validate_and_execute(&transaction).unwrap();
    let component_address = receipt.new_component_addresses[0];

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_method(component_address, "clear_vector", vec![])
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = sut.validate_and_execute(&transaction).unwrap();

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
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, true);
    let package = sut.publish_package(&compile("vault")).unwrap();
    let transaction = TransactionBuilder::new(&sut)
        .call_function(package, "VaultTest", "new_vault_into_vector", vec![])
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = sut.validate_and_execute(&transaction).unwrap();
    let component_address = receipt.new_component_addresses[0];

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_method(component_address, "push_vault_into_vector", vec![])
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = sut.validate_and_execute(&transaction).unwrap();

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn create_mutable_vault_with_take() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, true);
    let package = sut.publish_package(&compile("vault")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_function(package, "VaultTest", "new_vault_with_take", vec![])
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = sut.validate_and_execute(&transaction).unwrap();

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn create_mutable_vault_with_take_non_fungible() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, true);
    let package = sut.publish_package(&compile("vault")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_function(
            package,
            "VaultTest",
            "new_vault_with_take_non_fungible",
            vec![],
        )
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = sut.validate_and_execute(&transaction).unwrap();

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn create_mutable_vault_with_get_nonfungible_ids() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, true);
    let package = sut.publish_package(&compile("vault")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_function(
            package,
            "VaultTest",
            "new_vault_with_get_non_fungible_ids",
            vec![],
        )
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = sut.validate_and_execute(&transaction).unwrap();

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn create_mutable_vault_with_get_amount() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, true);
    let package = sut.publish_package(&compile("vault")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_function(package, "VaultTest", "new_vault_with_get_amount", vec![])
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = sut.validate_and_execute(&transaction).unwrap();

    // Assert
    receipt.result.expect("Should be okay");
}

#[test]
fn create_mutable_vault_with_get_resource_def() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut sut = TransactionExecutor::new(&mut ledger, true);
    let package = sut.publish_package(&compile("vault")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&sut)
        .call_function(
            package,
            "VaultTest",
            "new_vault_with_get_resource_def",
            vec![],
        )
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = sut.validate_and_execute(&transaction).unwrap();

    // Assert
    receipt.result.expect("Should be okay");
}
