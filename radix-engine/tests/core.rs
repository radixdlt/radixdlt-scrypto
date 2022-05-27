use radix_engine::ledger::*;
use radix_engine::model::extract_package;
use radix_engine::transaction::*;
use scrypto::call_data;
use scrypto::prelude::*;

#[test]
fn test_process_and_transaction() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let package = extract_package(compile_package!(format!("./tests/{}", "core"))).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    let transaction1 = TransactionBuilder::new()
        .call_function(package_address, "CoreTest", call_data![query()])
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt1 = executor.validate_and_execute(&transaction1).unwrap();
    receipt1.result.expect("Should be okay.");
}

#[test]
fn test_call() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (_, _, account) = executor.new_account();
    let package = extract_package(compile_package!(format!("./tests/{}", "core"))).unwrap();
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
