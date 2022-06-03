use radix_engine::engine::TransactionExecutor;
use radix_engine::ledger::*;
use radix_engine::model::extract_package;
use radix_engine::wasm::default_wasm_engine;
use scrypto::prelude::*;
use scrypto::to_struct;
use transaction::builder::TransactionBuilder;

#[test]
fn test_process_and_transaction() {
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);
    let package = extract_package(compile_package!(format!("./tests/{}", "core"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    let transaction1 = TransactionBuilder::new()
        .call_function(package_address, "CoreTest", "query", to_struct![])
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt1 = executor.validate_and_execute(&transaction1).unwrap();
    receipt1.result.expect("Should be okay.");
}

#[test]
fn test_call() {
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);
    let (_, _, account) = executor.new_account();
    let package = extract_package(compile_package!(format!("./tests/{}", "core"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    let transaction = TransactionBuilder::new()
        .call_function(package_address, "MoveTest", "move_bucket", to_struct![])
        .call_function(package_address, "MoveTest", "move_proof", to_struct![])
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();
    receipt.result.expect("Should be okay.");
}
