use radix_engine::errors::RuntimeError;
use radix_engine::ledger::*;
use radix_engine::model::ResourceDefError::InvalidFlagUpdate;
use radix_engine::model::{BucketError, ResourceContainerError, ResourceDefError};
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
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(&compile("resource_def")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "ResourceTest",
            "create_fungible",
            vec![],
            Some(account),
        )
        .call_function(package, "ResourceTest", "query", vec![], Some(account))
        .call_function(package, "ResourceTest", "burn", vec![], Some(account))
        .call_function(
            package,
            "ResourceTest",
            "update_feature_flags",
            vec![],
            Some(account),
        )
        .call_function(
            package,
            "ResourceTest",
            "update_resource_metadata",
            vec![],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();

    // Assert
    println!("{:?}", receipt);
    assert!(receipt.result.is_ok());
}

#[test]
fn take_with_bad_granularity_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(&compile("resource_def")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "ResourceTest",
            "create_fungible_should_fail",
            vec![],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    assert_eq!(
        runtime_error,
        RuntimeError::BucketError(BucketError::ResourceContainerError(
            ResourceContainerError::InvalidAmount(Decimal::from("0.1"), 0)
        ))
    );
}

#[test]
fn update_feature_flags_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(&compile("resource_def")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "ResourceTest",
            "update_feature_flags_should_fail",
            vec![],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    assert_eq!(
        runtime_error,
        RuntimeError::ResourceDefError(InvalidFlagUpdate {
            flags: 0,
            mutable_flags: 0,
            new_flags: MINTABLE,
            new_mutable_flags: 0,
        })
    );
}

#[test]
fn create_fungible_with_bad_resource_flags_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(&compile("resource_def")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "ResourceTest",
            "create_fungible_wrong_resource_flags_should_fail",
            vec![],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    assert_eq!(
        runtime_error,
        RuntimeError::ResourceDefError(ResourceDefError::InvalidResourceFlags(MAY_MINT | BURNABLE))
    );
}

#[test]
fn create_fungible_with_bad_mutable_flags_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(&compile("resource_def")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "ResourceTest",
            "create_fungible_wrong_mutable_flags_should_fail",
            vec![],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    assert_eq!(
        runtime_error,
        RuntimeError::ResourceDefError(ResourceDefError::InvalidResourceFlags(MAY_MINT))
    );
}

#[test]
fn create_fungible_with_bad_resource_permissions_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(&compile("resource_def")).unwrap();

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "ResourceTest",
            "create_fungible_wrong_resource_permissions_should_fail",
            vec![],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    assert_eq!(
        runtime_error,
        RuntimeError::ResourceDefError(ResourceDefError::InvalidResourcePermission(
            MINTABLE | MAY_BURN
        ))
    );
}
