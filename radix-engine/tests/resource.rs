use radix_engine::engine::RuntimeError;
use radix_engine::ledger::*;
use radix_engine::model::ResourceManagerError;
use radix_engine::transaction::*;
use radix_engine::wasm::default_wasm_engine;
use scrypto::call_data;
use scrypto::prelude::*;

#[test]
fn test_resource_manager() {
    // Arrange
    let mut ledger = InMemorySubstateStore::new();
    let wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut ledger, wasm_engine, true);
    let (pk, sk, account) = executor.new_account();
    let package = executor
        .publish_package(&compile_package!(format!("./tests/{}", "resource")))
        .unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(package, "ResourceTest", call_data!(create_fungible()))
        .call_function(package, "ResourceTest", call_data!(query()))
        .call_function(package, "ResourceTest", call_data!(burn()))
        .call_function(
            package,
            "ResourceTest",
            call_data!(update_resource_metadata()),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    println!("{:?}", receipt);
    assert!(receipt.result.is_ok());
}

#[test]
fn mint_with_bad_granularity_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::new();
    let wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut ledger, wasm_engine, true);
    let (pk, sk, account) = executor.new_account();
    let package = executor
        .publish_package(&compile_package!(format!("./tests/{}", "resource")))
        .unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(
            package,
            "ResourceTest",
            call_data![create_fungible_and_mint(0u8, dec!("0.1"))],
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
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
    let mut ledger = InMemorySubstateStore::new();
    let wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut ledger, wasm_engine, true);
    let (pk, sk, account) = executor.new_account();
    let package = executor
        .publish_package(&compile_package!(format!("./tests/{}", "resource")))
        .unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(
            package,
            "ResourceTest",
            call_data![create_fungible_and_mint(0u8, dec!(100_000_000_001i128))],
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    assert_eq!(
        runtime_error,
        RuntimeError::ResourceManagerError(ResourceManagerError::MaxMintAmountExceeded)
    );
}
