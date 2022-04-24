#[rustfmt::skip]
pub mod test_runner;

use scrypto::args_untyped;
use crate::test_runner::TestRunner;
use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn create_non_fungible_mutable() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (_, _, account) = test_runner.new_account();
    let package = test_runner.publish_package("non_fungible");

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_function(
            package,
            "NonFungibleTest",
            args_untyped!(create_non_fungible_mutable()),
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
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, sk, account) = test_runner.new_account();
    let package = test_runner.publish_package("non_fungible");
    let transaction = test_runner
        .new_transaction_builder()
        .call_function(
            package,
            "NonFungibleTest",
            args_untyped!(create_burnable_non_fungible()),
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
            args_untyped![verify_does_not_exist(non_fungible_address)],
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
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (pk, sk, account) = executor.new_account();
    let package = executor
        .publish_package(&compile_package!(format!("./tests/{}", "non_fungible")))
        .unwrap();

    let transaction = TransactionBuilder::new()
        .call_function(
            package,
            "NonFungibleTest",
            args_untyped!(create_non_fungible_fixed()),
        )
        .call_function(
            package,
            "NonFungibleTest",
            args_untyped!(update_and_get_non_fungible()),
        )
        .call_function(package, "NonFungibleTest", args_untyped!(non_fungible_exists()))
        .call_function(package, "NonFungibleTest", args_untyped!(take_and_put_bucket()))
        .call_function(package, "NonFungibleTest", args_untyped!(take_and_put_vault()))
        .call_function(
            package,
            "NonFungibleTest",
            args_untyped!(get_non_fungible_ids_bucket()),
        )
        .call_function(
            package,
            "NonFungibleTest",
            args_untyped!(get_non_fungible_ids_vault()),
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
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (pk, sk, account) = executor.new_account();
    let package = executor
        .publish_package(&compile_package!(format!("./tests/{}", "non_fungible")))
        .unwrap();

    let transaction = TransactionBuilder::new()
        .call_function(package, "NonFungibleTest", args_untyped!(singleton_non_fungible()))
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();
    println!("{:?}", receipt);
    assert!(receipt.result.is_ok());
}
