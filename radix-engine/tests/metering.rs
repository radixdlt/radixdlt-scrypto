#[rustfmt::skip]
pub mod test_runner;

use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::call_data;
use scrypto::prelude::*;

#[test]
fn test_loop() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let package = executor
        .publish_package(&compile_package!(format!("./tests/{}", "metering")))
        .unwrap();

    let transaction = TransactionBuilder::new()
        .call_function(package, "Metering", call_data!(loooop(100u32)))
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();
    assert!(receipt.result.is_ok());
}

#[test]
fn test_loop_out_of_tbd() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let package = executor
        .publish_package(&compile_package!(format!("./tests/{}", "metering")))
        .unwrap();

    let transaction = TransactionBuilder::new()
        .call_function(package, "Metering", call_data!(loooop(100u32)))
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();
    assert!(receipt.result.is_ok());
}

#[test]
fn test_fib() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let package = executor
        .publish_package(&compile_package!(format!("./tests/{}", "metering")))
        .unwrap();

    let transaction = TransactionBuilder::new()
        .call_function(package, "Metering", call_data!(loooop(100u32)))
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();
    assert!(receipt.result.is_ok());
}

#[test]
fn test_fib_out_of_stack() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let package = executor
        .publish_package(&compile_package!(format!("./tests/{}", "metering")))
        .unwrap();

    let transaction = TransactionBuilder::new()
        .call_function(package, "Metering", call_data!(loooop(100u32)))
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();
    assert!(receipt.result.is_ok());
}
