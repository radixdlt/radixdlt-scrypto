use radix_engine::errors::RuntimeError;
use radix_engine::ledger::*;
use radix_engine::model::ResourceDefError;
use radix_engine::transaction::*;
use scrypto::prelude::*;

pub fn compile(name: &str) -> Vec<u8> {
    compile_package!(format!("./tests/{}", name), name.replace("-", "_"))
}

#[test]
fn test_resource_def() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (key, account) = executor.new_public_key_with_account();
    let package = executor.publish_package(&compile("resource")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .call_function(package, "ResourceTest", "create_fungible", vec![])
        .call_function(package, "ResourceTest", "query", vec![])
        .call_function(package, "ResourceTest", "burn", vec![])
        .call_function(package, "ResourceTest", "update_resource_metadata", vec![])
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();

    // Assert
    println!("{:?}", receipt);
    assert!(receipt.result.is_ok());
}

#[test]
fn mint_with_bad_granularity_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (key, account) = executor.new_public_key_with_account();
    let package = executor.publish_package(&compile("resource")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "ResourceTest",
            "create_fungible_and_mint",
            args![0u8, dec!("0.1")],
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    assert_eq!(
        runtime_error,
        RuntimeError::ResourceDefError(ResourceDefError::InvalidAmount(Decimal::from("0.1"), 0))
    );
}

#[test]
fn mint_too_much_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (key, account) = executor.new_public_key_with_account();
    let package = executor.publish_package(&compile("resource")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "ResourceTest",
            "create_fungible_and_mint",
            args![0u8, dec!(100_000_000_001i128)],
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    assert_eq!(
        runtime_error,
        RuntimeError::ResourceDefError(ResourceDefError::MaxMintAmountExceeded)
    );
}
