#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::ledger::*;
use radix_engine::transaction::*;
use radix_engine::wasm::default_wasm_engine;
use scrypto::call_data;
use scrypto::prelude::*;

#[test]
fn create_non_fungible_mutable() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut test_runner = TestRunner::new(&mut substate_store, default_wasm_engine());
    let (_, _, account) = test_runner.new_account();
    let package = test_runner.publish_package("non_fungible");

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_function(
            package,
            "NonFungibleTest",
            call_data!(create_non_fungible_mutable()),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn can_burn_non_fungible() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut test_runner = TestRunner::new(&mut substate_store, default_wasm_engine());
    let (pk, sk, account) = test_runner.new_account();
    let package = test_runner.publish_package("non_fungible");
    let transaction = test_runner
        .new_transaction_builder()
        .call_function(
            package,
            "NonFungibleTest",
            call_data!(create_burnable_non_fungible()),
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
            call_data![verify_does_not_exist(non_fungible_address)],
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
    let mut ledger = InMemorySubstateStore::new();
    let mut executor = TransactionExecutor::new(&mut ledger, default_wasm_engine(), true);
    let (pk, sk, account) = executor.new_account();
    let package = executor
        .publish_package(&compile_package!(format!("./tests/{}", "non_fungible")))
        .unwrap();

    let transaction = TransactionBuilder::new()
        .call_function(
            package,
            "NonFungibleTest",
            call_data!(create_non_fungible_fixed()),
        )
        .call_function(
            package,
            "NonFungibleTest",
            call_data!(update_and_get_non_fungible()),
        )
        .call_function(
            package,
            "NonFungibleTest",
            call_data!(non_fungible_exists()),
        )
        .call_function(
            package,
            "NonFungibleTest",
            call_data!(take_and_put_bucket()),
        )
        .call_function(package, "NonFungibleTest", call_data!(take_and_put_vault()))
        .call_function(
            package,
            "NonFungibleTest",
            call_data!(get_non_fungible_ids_bucket()),
        )
        .call_function(
            package,
            "NonFungibleTest",
            call_data!(get_non_fungible_ids_vault()),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();
    println!("{:?}", receipt);
    assert!(receipt.result.is_ok());
}

#[test]
fn test_singleton_non_fungible() {
    let mut ledger = InMemorySubstateStore::new();
    let mut executor = TransactionExecutor::new(&mut ledger, default_wasm_engine(), true);
    let (pk, sk, account) = executor.new_account();
    let package = executor
        .publish_package(&compile_package!(format!("./tests/{}", "non_fungible")))
        .unwrap();

    let transaction = TransactionBuilder::new()
        .call_function(
            package,
            "NonFungibleTest",
            call_data!(singleton_non_fungible()),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();
    println!("{:?}", receipt);
    assert!(receipt.result.is_ok());
}
