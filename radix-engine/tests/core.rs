use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

pub fn compile(name: &str) -> Vec<u8> {
    compile_package!(format!("./tests/{}", name), name.replace("-", "_"))
}

#[test]
fn test_process_and_transaction() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (_, account) = executor.new_public_key_with_account();
    let package = executor.publish_package(&compile("core")).unwrap();

    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "CoreTest", "query", vec![], Some(account))
        .build(vec![])
        .unwrap();
    let receipt1 = executor.run(transaction1).unwrap();
    assert!(receipt1.result.is_ok());
}

#[test]
fn test_call() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (key, account) = executor.new_public_key_with_account();
    let package = executor.publish_package(&compile("core")).unwrap();

    let transaction = TransactionBuilder::new(&executor)
        .call_function(package, "MoveTest", "move_bucket", vec![], Some(account))
        .call_function(package, "MoveTest", "move_proof", vec![], Some(account))
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();
    assert!(receipt.result.is_ok());
}
