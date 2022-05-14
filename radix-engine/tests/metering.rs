#[rustfmt::skip]
pub mod test_runner;

use radix_engine::ledger::*;
use radix_engine::transaction::*;
use radix_engine::wasm::default_wasm_engine;
use radix_engine::wasm::InvokeError;
use scrypto::call_data;
use scrypto::prelude::*;

#[test]
fn test_loop() {
    let mut ledger = InMemorySubstateStore::new();
    let wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut ledger, wasm_engine, true);
    let package = executor
        .publish_package(&compile_package!(format!("./tests/{}", "metering")))
        .unwrap();

    let transaction = TransactionBuilder::new()
        .call_function(package, "Metering", call_data!(iterations(10_000u32)))
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();
    assert!(receipt.result.is_ok());
}

#[test]
fn test_loop_out_of_tbd() {
    let mut ledger = InMemorySubstateStore::new();
    let wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut ledger, wasm_engine, true);
    let package = executor
        .publish_package(&compile_package!(format!("./tests/{}", "metering")))
        .unwrap();

    let transaction = TransactionBuilder::new()
        .call_function(package, "Metering", call_data!(iterations(5_000_000u32)))
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();
    assert_invoke_error!(receipt.result, InvokeError::OutOfTbd { .. });
}
