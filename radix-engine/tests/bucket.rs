#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::errors::*;
use radix_engine::ledger::*;
use radix_engine::model::ResourceContainerError;
use radix_engine::transaction::*;
use scrypto::prelude::*;

pub fn compile(name: &str) -> Vec<u8> {
    compile_package!(format!("./tests/{}", name), name.replace("-", "_"))
}

#[test]
fn test_bucket() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (_, _, account) = executor.new_account();
    let package = executor.publish_package(&compile("bucket")).unwrap();

    let transaction = TransactionBuilder::new(&executor)
        .call_function(package, "BucketTest", "combine", vec![])
        .call_function(package, "BucketTest", "split", vec![])
        .call_function(package, "BucketTest", "borrow", vec![])
        .call_function(package, "BucketTest", "query", vec![])
        .call_function(package, "BucketTest", "test_restricted_transfer", vec![])
        .call_function(package, "BucketTest", "test_burn", vec![])
        .call_function(package, "BucketTest", "test_burn_freely", vec![])
        .call_method_with_all_resources(account, "deposit_batch")
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = executor.validate_and_execute(&transaction).unwrap();
    assert!(receipt.result.is_ok());
}

#[test]
fn test_bucket_of_badges() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (_, _, account) = executor.new_account();
    let package = executor.publish_package(&compile("bucket")).unwrap();

    let transaction = TransactionBuilder::new(&executor)
        .call_function(package, "BadgeTest", "combine", vec![])
        .call_function(package, "BadgeTest", "split", vec![])
        .call_function(package, "BadgeTest", "borrow", vec![])
        .call_function(package, "BadgeTest", "query", vec![])
        .call_method_with_all_resources(account, "deposit_batch")
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = executor.validate_and_execute(&transaction).unwrap();
    assert!(receipt.result.is_ok());
}

#[test]
fn test_take_with_invalid_granularity() {
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, sk, account) = test_runner.new_account();
    let resource_def_id = test_runner.create_fungible_resource(100.into(), 2, account);
    let package_id = test_runner.publish_package("bucket");

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .parse_args_and_call_function(
            package_id,
            "BucketTest",
            "take_from_bucket",
            vec![format!("100,{}", resource_def_id), "1.123".to_owned()],
            Some(account),
        )
        .build_and_sign(vec![pk], vec![sk])
        .unwrap();
    let receipt = test_runner.validate_and_execute(&transaction);
    println!("{:?}", receipt);

    // Assert
    assert_eq!(
        receipt.result,
        Err(RuntimeError::BucketError(
            ResourceContainerError::InvalidAmount(dec!("1.123"), 2)
        ))
    );
}

#[test]
fn test_take_with_negative_amount() {
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, sk, account) = test_runner.new_account();
    let resource_def_id = test_runner.create_fungible_resource(100.into(), 2, account);
    let package_id = test_runner.publish_package("bucket");

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .parse_args_and_call_function(
            package_id,
            "BucketTest",
            "take_from_bucket",
            vec![format!("100,{}", resource_def_id), "-2".to_owned()],
            Some(account),
        )
        .build_and_sign(vec![pk], vec![sk])
        .unwrap();
    let receipt = test_runner.validate_and_execute(&transaction);
    println!("{:?}", receipt);

    // Assert
    assert_eq!(
        receipt.result,
        Err(RuntimeError::BucketError(
            ResourceContainerError::InvalidAmount(dec!("-2"), 2)
        ))
    );
}
