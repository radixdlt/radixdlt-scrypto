use radix_engine::ledger::*;
use radix_engine::transaction::*;
use radix_engine::wasm::default_wasm_engine;
use scrypto::call_data;
use scrypto::prelude::*;

#[test]
fn test_process_and_transaction() {
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);
    let package_address = executor
        .publish_package(Package::new(compile_package!(format!(
            "./tests/{}",
            "core"
        ))))
        .unwrap();

    let transaction1 = TransactionBuilder::new()
        .call_function(package_address, "CoreTest", call_data![query()])
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt1 = executor.validate_and_execute(&transaction1).unwrap();
    receipt1.result.expect("Should be okay.");
}

#[test]
fn test_call() {
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);
    let (_, _, account) = executor.new_account();
    let package = Package::new(compile_package!(format!("./tests/{}", "core")));
    let package_address = executor.publish_package(package).unwrap();

    let transaction = TransactionBuilder::new()
        .call_function(package_address, "MoveTest", call_data![move_bucket()])
        .call_function(package_address, "MoveTest", call_data![move_proof()])
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();
    receipt.result.expect("Should be okay.");
}
