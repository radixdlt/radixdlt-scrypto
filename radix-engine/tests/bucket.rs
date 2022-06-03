#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::engine::*;
use radix_engine::ledger::*;
use radix_engine::model::{extract_package, BucketError, ResourceContainerError};
use radix_engine::transaction::*;
use radix_engine::wasm::default_wasm_engine;
use scrypto::prelude::*;
use scrypto::to_struct;

#[test]
fn test_bucket() {
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);
    let (_, _, account) = executor.new_account();
    let package = extract_package(compile_package!(format!("./tests/{}", "bucket"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    let transaction = TransactionBuilder::new()
        .call_function(package_address, "BucketTest", "combine", to_struct!())
        .call_function(package_address, "BucketTest", "split", to_struct!())
        .call_function(package_address, "BucketTest", "borrow", to_struct!())
        .call_function(package_address, "BucketTest", "query", to_struct!())
        .call_function(
            package_address,
            "BucketTest",
            "test_restricted_transfer",
            to_struct!(),
        )
        .call_function(package_address, "BucketTest", "test_burn", to_struct!())
        .call_function(
            package_address,
            "BucketTest",
            "test_burn_freely",
            to_struct!(),
        )
        .call_function(
            package_address,
            "BucketTest",
            "create_empty_bucket_fungible",
            to_struct!(),
        )
        .call_function(
            package_address,
            "BucketTest",
            "create_empty_bucket_non_fungible",
            to_struct!(),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();
    receipt.result.expect("It should work");
}

#[test]
fn test_bucket_of_badges() {
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);
    let (_, _, account) = executor.new_account();
    let package = extract_package(compile_package!(format!("./tests/{}", "bucket"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    let transaction = TransactionBuilder::new()
        .call_function(package_address, "BadgeTest", "combine", to_struct!())
        .call_function(package_address, "BadgeTest", "split", to_struct!())
        .call_function(package_address, "BadgeTest", "borrow", to_struct!())
        .call_function(package_address, "BadgeTest", "query", to_struct!())
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();
    receipt.result.expect("It should work");
}

#[test]
fn test_take_with_invalid_granularity() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut wasm_engine = default_wasm_engine();
    let mut test_runner = TestRunner::new(&mut substate_store, &mut wasm_engine);
    let (pk, sk, account) = test_runner.new_account();
    let resource_address = test_runner.create_fungible_resource(100.into(), 2, account);
    let package_address = test_runner.publish_package("bucket");

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_function_with_abi(
            package_address,
            "BucketTest",
            "take_from_bucket",
            vec![format!("100,{}", resource_address), "1.123".to_owned()],
            Some(account),
            &test_runner.export_abi(package_address, "BucketTest"),
        )
        .unwrap()
        .build(test_runner.get_nonce([pk]))
        .sign([&sk]);
    let receipt = test_runner.validate_and_execute(&transaction);
    println!("{:?}", receipt);

    // Assert
    assert_eq!(
        receipt.result,
        Err(RuntimeError::BucketError(
            BucketError::ResourceContainerError(ResourceContainerError::InvalidAmount(
                dec!("1.123"),
                2
            ))
        ))
    );
}

#[test]
fn test_take_with_negative_amount() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut wasm_engine = default_wasm_engine();
    let mut test_runner = TestRunner::new(&mut substate_store, &mut wasm_engine);
    let (pk, sk, account) = test_runner.new_account();
    let resource_address = test_runner.create_fungible_resource(100.into(), 2, account);
    let package_address = test_runner.publish_package("bucket");

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_function_with_abi(
            package_address,
            "BucketTest",
            "take_from_bucket",
            vec![format!("100,{}", resource_address), "-2".to_owned()],
            Some(account),
            &test_runner.export_abi(package_address, "BucketTest"),
        )
        .unwrap()
        .build(test_runner.get_nonce([pk]))
        .sign([&sk]);
    let receipt = test_runner.validate_and_execute(&transaction);
    println!("{:?}", receipt);

    // Assert
    assert_eq!(
        receipt.result,
        Err(RuntimeError::BucketError(
            BucketError::ResourceContainerError(ResourceContainerError::InvalidAmount(
                dec!("-2"),
                2
            ))
        ))
    );
}

#[test]
fn create_empty_bucket() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut wasm_engine = default_wasm_engine();
    let mut test_runner = TestRunner::new(&mut substate_store, &mut wasm_engine);
    let (pk, sk, account) = test_runner.new_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .take_from_worktop(scrypto::prelude::RADIX_TOKEN, |builder, _bucket_id| builder)
        .take_from_worktop_by_amount(
            Decimal::zero(),
            scrypto::prelude::RADIX_TOKEN,
            |builder, _bucket_id| builder,
        )
        .take_from_worktop_by_ids(
            &BTreeSet::new(),
            scrypto::prelude::RADIX_TOKEN,
            |builder, _bucket_id| builder,
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(test_runner.get_nonce([pk]))
        .sign([&sk]);
    let receipt = test_runner.validate_and_execute(&transaction);
    println!("{:?}", receipt);

    // Assert
    receipt.result.expect("It should work");
}
