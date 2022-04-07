use radix_engine::errors::RuntimeError;
use radix_engine::ledger::*;
use radix_engine::model::ResourceManagerError;
use radix_engine::model::ResourceManagerError::FlagsLocked;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_resource_manager() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (pk, sk, account) = executor.new_account();
    let package = executor
        .publish_package(&compile_package!(format!("./tests/{}", "resource")))
        .unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(package, "ResourceTest", "create_fungible", args![])
        .call_function(package, "ResourceTest", "query", args![])
        .call_function(package, "ResourceTest", "burn", args![])
        .call_function(package, "ResourceTest", "update_feature_flags", args![])
        .call_function(package, "ResourceTest", "update_resource_metadata", args![])
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce(&[pk]))
        .sign(&[sk]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    println!("{:?}", receipt);
    assert!(receipt.result.is_ok());
}

#[test]
fn mint_with_bad_granularity_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (pk, sk, account) = executor.new_account();
    let package = executor
        .publish_package(&compile_package!(format!("./tests/{}", "resource")))
        .unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(
            package,
            "ResourceTest",
            "create_fungible_and_mint",
            args![0u8, dec!("0.1")],
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce(&[pk]))
        .sign(&[sk]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    assert_eq!(
        runtime_error,
        RuntimeError::ResourceManagerError(ResourceManagerError::InvalidAmount(
            Decimal::from("0.1"),
            0
        ))
    );
}

#[test]
fn mint_too_much_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (pk, sk, account) = executor.new_account();
    let package = executor
        .publish_package(&compile_package!(format!("./tests/{}", "resource")))
        .unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(
            package,
            "ResourceTest",
            "create_fungible_and_mint",
            args![0u8, dec!(100_000_000_001i128)],
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce(&[pk]))
        .sign(&[sk]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    assert_eq!(
        runtime_error,
        RuntimeError::ResourceManagerError(ResourceManagerError::MaxMintAmountExceeded)
    );
}

#[test]
fn update_feature_flags_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (pk, sk, account) = executor.new_account();
    let package = executor
        .publish_package(&compile_package!(format!("./tests/{}", "resource")))
        .unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(
            package,
            "ResourceTest",
            "update_feature_flags_should_fail",
            vec![],
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce(&[pk]))
        .sign(&[sk]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    assert_eq!(
        runtime_error,
        RuntimeError::ResourceManagerError(FlagsLocked)
    );
}

#[test]
fn create_fungible_with_bad_resource_flags_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (pk, sk, account) = executor.new_account();
    let package = executor
        .publish_package(&compile_package!(format!("./tests/{}", "resource")))
        .unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(
            package,
            "ResourceTest",
            "create_fungible_wrong_resource_flags_should_fail",
            vec![],
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce(&[pk]))
        .sign(&[sk]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    assert_eq!(
        runtime_error,
        RuntimeError::ResourceManagerError(ResourceManagerError::InvalidResourceFlags(
            MAY_MINT | BURNABLE
        ))
    );
}

#[test]
fn create_fungible_with_bad_mutable_flags_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (pk, sk, account) = executor.new_account();
    let package = executor
        .publish_package(&compile_package!(format!("./tests/{}", "resource")))
        .unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(
            package,
            "ResourceTest",
            "create_fungible_wrong_mutable_flags_should_fail",
            vec![],
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce(&[pk]))
        .sign(&[sk]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    assert_eq!(
        runtime_error,
        RuntimeError::ResourceManagerError(ResourceManagerError::InvalidResourceFlags(MAY_MINT))
    );
}

#[test]
fn create_fungible_with_bad_resource_permissions_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (pk, sk, account) = executor.new_account();
    let package = executor
        .publish_package(&compile_package!(format!("./tests/{}", "resource")))
        .unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(
            package,
            "ResourceTest",
            "create_fungible_wrong_resource_permissions_should_fail",
            vec![],
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce(&[pk]))
        .sign(&[sk]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    assert_eq!(
        runtime_error,
        RuntimeError::ResourceManagerError(ResourceManagerError::InvalidResourcePermission(
            MINTABLE | MAY_BURN
        ))
    );
}
