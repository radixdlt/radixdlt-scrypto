#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::ledger::*;
use radix_engine::model::extract_package;
use radix_engine::transaction::*;
use radix_engine::wasm::default_wasm_engine;
use scrypto::prelude::*;
use scrypto::to_struct;

#[test]
fn create_non_fungible_mutable() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut test_runner = TestRunner::new(&mut substate_store, &mut wasm_engine);
    let (_, _, account) = test_runner.new_account();
    let package = test_runner.publish_package("non_fungible");

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_function(
            package,
            "NonFungibleTest",
            "create_non_fungible_mutable",
            to_struct!(),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    receipt.result.expect("It should work");
}

#[test]
fn can_burn_non_fungible() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut test_runner = TestRunner::new(&mut substate_store, &mut wasm_engine);
    let (pk, sk, account) = test_runner.new_account();
    let package = test_runner.publish_package("non_fungible");
    let transaction = test_runner
        .new_transaction_builder()
        .call_function(
            package,
            "NonFungibleTest",
            "create_burnable_non_fungible",
            to_struct!(),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt = test_runner.validate_and_execute(&transaction);
    receipt.result.expect("Should be okay.");
    let resource_address = receipt.new_resource_addresses[0];
    let non_fungible_address =
        NonFungibleAddress::new(resource_address, NonFungibleId::from_u32(0));
    let mut ids = BTreeSet::new();
    ids.insert(NonFungibleId::from_u32(0));

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(resource_address, account)
        .burn_non_fungible(non_fungible_address.clone())
        .call_function(
            package,
            "NonFungibleTest",
            "verify_does_not_exist",
            to_struct!(non_fungible_address),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(test_runner.get_nonce([pk]))
        .sign([&sk]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    receipt.result.expect("Should be okay.");
}

#[test]
fn test_non_fungible() {
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);
    let (pk, sk, account) = executor.new_account();
    let package = extract_package(compile_package!(format!("./tests/{}", "non_fungible"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    let transaction = TransactionBuilder::new()
        .call_function(
            package_address,
            "NonFungibleTest",
            "create_non_fungible_fixed",
            to_struct!(),
        )
        .call_function(
            package_address,
            "NonFungibleTest",
            "update_and_get_non_fungible",
            to_struct!(),
        )
        .call_function(
            package_address,
            "NonFungibleTest",
            "non_fungible_exists",
            to_struct!(),
        )
        .call_function(
            package_address,
            "NonFungibleTest",
            "take_and_put_bucket",
            to_struct!(),
        )
        .call_function(
            package_address,
            "NonFungibleTest",
            "take_and_put_vault",
            to_struct!(),
        )
        .call_function(
            package_address,
            "NonFungibleTest",
            "get_non_fungible_ids_bucket",
            to_struct!(),
        )
        .call_function(
            package_address,
            "NonFungibleTest",
            "get_non_fungible_ids_vault",
            to_struct!(),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();
    println!("{:?}", receipt);
    receipt.result.expect("It should work");
}

#[test]
fn test_singleton_non_fungible() {
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);
    let (pk, sk, account) = executor.new_account();
    let package = extract_package(compile_package!(format!("./tests/{}", "non_fungible"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    let transaction = TransactionBuilder::new()
        .call_function(
            package_address,
            "NonFungibleTest",
            "singleton_non_fungible",
            to_struct!(),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();
    println!("{:?}", receipt);
    receipt.result.expect("It should work");
}
