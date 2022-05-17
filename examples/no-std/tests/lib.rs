// This is optional, as you may choose to use std for testing only.
#![no_std]

use radix_engine::ledger::*;
use radix_engine::transaction::*;
use radix_engine::wasm::default_wasm_engine;
use scrypto::call_data;
use scrypto::prelude::*;

#[test]
fn test_say_hello() {
    // Set up environment.
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, false);
    let package = executor
        .publish_package(include_package!("no_std"))
        .unwrap();

    // Test the `say_hello` function.
    let transaction1 = TransactionBuilder::new()
        .call_function(package, "NoStd", call_data!(say_hello()))
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt1 = executor.validate_and_execute(&transaction1).unwrap();
    assert!(receipt1.result.is_ok());
}
