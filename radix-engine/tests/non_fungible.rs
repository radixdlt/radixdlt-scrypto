#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

pub fn compile(name: &str) -> Vec<u8> {
    compile_package!(format!("./tests/{}", name), name.replace("-", "_"))
}

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
            "create_non_fungible_mutable",
            vec![],
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(test_runner.get_nonce(&[]))
        .sign(&[]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn test_non_fungible() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (pk, sk, account) = executor.new_account();
    let package = executor.publish_package(&compile("non_fungible")).unwrap();

    let transaction = TransactionBuilder::new()
        .call_function(
            package,
            "NonFungibleTest",
            "create_non_fungible_fixed",
            vec![],
        )
        .call_function(
            package,
            "NonFungibleTest",
            "update_and_get_non_fungible",
            vec![],
        )
        .call_function(package, "NonFungibleTest", "non_fungible_exists", vec![])
        .call_function(package, "NonFungibleTest", "take_and_put_bucket", vec![])
        .call_function(package, "NonFungibleTest", "take_and_put_vault", vec![])
        .call_function(
            package,
            "NonFungibleTest",
            "get_non_fungible_ids_bucket",
            vec![],
        )
        .call_function(
            package,
            "NonFungibleTest",
            "get_non_fungible_ids_vault",
            vec![],
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce(&[pk]))
        .sign(&[sk]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();
    println!("{:?}", receipt);
    assert!(receipt.result.is_ok());
}
