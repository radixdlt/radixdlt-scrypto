// This is optional, as you may choose to use std for testing only.
#![no_std]

use radix_engine::ledger::*;
use radix_engine::model::extract_package;
use radix_engine::transaction::*;
use scrypto::call_data;
use scrypto::prelude::*;

#[test]
fn test_say_hello() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let package = extract_package(include_package!("no_std").to_vec()).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    // Test the `say_hello` function.
    let transaction1 = TransactionBuilder::new()
        .call_function(package_address, "NoStd", call_data!(say_hello()))
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt1 = executor.validate_and_execute(&transaction1).unwrap();
    assert!(receipt1.result.is_ok());
}
